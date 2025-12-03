// crates/ycg_core/src/lib.rs
pub mod adhoc_format;
pub mod adhoc_serializer_v2;
pub mod ast_cache;
pub mod config;
pub mod enricher;
pub mod errors;
pub mod file_filter;
pub mod framework_filter;
pub mod logic_extractor;
pub mod model;
pub mod semantic_filter;
pub mod signature_extractor;
pub mod type_abbreviator;
pub mod validators;

pub mod scip_proto {
    include!(concat!(env!("OUT_DIR"), "/scip.rs"));
}

use crate::enricher::TreeSitterEnricher;
use crate::model::{
    EdgeType, LogicMetadata, ProjectMetadata, ReferenceEdge, ScipSymbolKind, SymbolNode, YcgGraph,
    YcgGraphOptimized,
};
use anyhow::{Context, Result};
use prost::Message;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tiktoken_rs::cl100k_base;
use xxhash_rust::xxh64::xxh64;

// --- CONFIGURAÇÃO ---

#[derive(Debug, Clone, Copy)]
pub enum LevelOfDetail {
    Low,
    Medium,
    High,
}

pub struct YcgConfig {
    pub lod: LevelOfDetail,
    pub project_root: PathBuf,
    pub compact: bool, // Flag para ativar lista de adjacência

    // New fields for token optimization
    pub output_format: model::OutputFormat,
    pub ignore_framework_noise: bool,
    pub file_filter: model::FileFilterConfig,

    // Ad-hoc granularity level (Requirements 1.1-1.6)
    pub adhoc_granularity: model::AdHocGranularity,
}

struct Scope {
    id: u64,
    start_line: i32,
    end_line: i32,
}

pub fn count_tokens(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(text);
    tokens.len()
}

pub fn run_scip_conversion(scip_path: &Path, config: YcgConfig) -> Result<String> {
    if !scip_path.exists() {
        anyhow::bail!("Arquivo SCIP não encontrado: {:?}", scip_path);
    }
    println!("Carregando índice SCIP de: {:?}", scip_path);

    let data = fs::read(scip_path).with_context(|| format!("Falha ao ler: {:?}", scip_path))?;
    let mut index = scip_proto::Index::decode(&data[..]).context("Falha ao decodificar SCIP")?;

    // STEP 1: File Filtering (Requirements 4.1-4.7)
    // Apply file filtering before processing if any filters are configured
    if !config.file_filter.include_patterns.is_empty()
        || !config.file_filter.exclude_patterns.is_empty()
        || config.file_filter.use_gitignore
    {
        println!(">>> Aplicando filtros de arquivo...");
        let file_filter = file_filter::FileFilter::new(&config.file_filter, &config.project_root)?;
        let original_count = index.documents.len();
        index.documents = file_filter.filter_documents(index.documents);
        let filtered_count = index.documents.len();
        println!(
            "    Arquivos filtrados: {} -> {} ({} removidos)",
            original_count,
            filtered_count,
            original_count - filtered_count
        );
    }

    // Contagem de Tokens de Entrada
    let mut total_input_tokens = 0;
    let project_root = &config.project_root;

    for doc in &index.documents {
        let real_path = project_root.join(&doc.relative_path);
        if let Ok(content) = fs::read_to_string(&real_path) {
            total_input_tokens += count_tokens(&content);
        }
    }
    println!("--- Métrica de Densidade ---");
    println!("Input Total Tokens (Código Bruto): {}", total_input_tokens);

    // Build source code map for signature/logic extraction (before consuming index)
    let mut sources = HashMap::new();
    for doc in &index.documents {
        let real_path = project_root.join(&doc.relative_path);
        if let Ok(content) = fs::read_to_string(&real_path) {
            // Map file path to source content
            sources.insert(doc.relative_path.clone(), content);
        }
    }

    // Gera o grafo padrão (Flat)
    let mut graph = convert_scip_to_ycg(index, &config);

    // STEP 2: Semantic Filtering / Graph Compaction (Requirements 1.1-1.8)
    // Apply semantic filtering if compact mode is enabled
    if config.compact {
        println!(">>> Aplicando compactação semântica do grafo...");
        let original_nodes = graph.definitions.len();
        let original_edges = graph.references.len();
        semantic_filter::SemanticFilter::filter_graph(&mut graph);
        let filtered_nodes = graph.definitions.len();
        let filtered_edges = graph.references.len();
        println!(
            "    Nós: {} -> {} ({:.1}% redução)",
            original_nodes,
            filtered_nodes,
            (1.0 - filtered_nodes as f64 / original_nodes as f64) * 100.0
        );
        println!(
            "    Arestas: {} -> {} ({:.1}% redução)",
            original_edges,
            filtered_edges,
            (1.0 - filtered_edges as f64 / original_edges as f64) * 100.0
        );
    }

    // STEP 3: Framework Noise Reduction (Requirements 2.1-2.6)
    // Apply framework noise filtering if enabled
    if config.ignore_framework_noise {
        println!(">>> Removendo ruído de framework...");
        let original_nodes = graph.definitions.len();
        framework_filter::FrameworkNoiseFilter::filter_graph(&mut graph);
        let filtered_nodes = graph.definitions.len();
        println!(
            "    Nós após remoção de boilerplate: {} -> {} ({} removidos)",
            original_nodes,
            filtered_nodes,
            original_nodes - filtered_nodes
        );
    }

    // STEP 4: Format Selection (Requirements 3.1-3.5)
    // Serialize based on output format
    let output = match config.output_format {
        model::OutputFormat::AdHoc => {
            println!(">>> Serializando em formato Ad-Hoc...");

            // Log granularity level (Requirements 1.1-1.6)
            let granularity_str = match config.adhoc_granularity {
                model::AdHocGranularity::Default => "Level 0 (Default)",
                model::AdHocGranularity::InlineSignatures => "Level 1 (Inline Signatures)",
                model::AdHocGranularity::InlineLogic => "Level 2 (Inline Logic)",
            };
            println!("    Granularity: {}", granularity_str);

            // Use AdHocSerializerV2 with granularity support
            let serializer = adhoc_serializer_v2::AdHocSerializerV2::new(config.adhoc_granularity);
            let adhoc_graph = serializer.serialize_graph(&graph, &sources);
            serde_yaml::to_string(&adhoc_graph)?
        }
        model::OutputFormat::Yaml => {
            // Standard YAML format
            if config.compact {
                println!(">>> Otimizando Grafo: Aplicando Lista de Adjacência...");
                let optimized_graph = optimize_graph(graph);
                serde_yaml::to_string(&optimized_graph)?
            } else {
                serde_yaml::to_string(&graph)?
            }
        }
    };

    // Contagem de Tokens de Saída
    let output_tokens = count_tokens(&output);
    println!("Output Total Tokens (Grafo YAML): {}", output_tokens);

    if total_input_tokens > 0 {
        let ratio = total_input_tokens as f64 / output_tokens as f64;
        println!("Taxa de Compressão: {:.2}x", ratio);
    }
    println!("--------------------------");

    Ok(output)
}

// Transformador: Flat List -> Adjacency List
fn optimize_graph(graph: YcgGraph) -> YcgGraphOptimized {
    let mut adjacency: BTreeMap<String, BTreeMap<EdgeType, Vec<String>>> = BTreeMap::new();

    for edge in graph.references {
        // Pega ou cria o mapa para este nó de origem
        let node_edges = adjacency.entry(edge.from).or_insert_with(BTreeMap::new);

        // Pega ou cria a lista para este tipo de aresta
        let targets = node_edges.entry(edge.edge_type).or_insert_with(Vec::new);

        // Adiciona o destino
        targets.push(edge.to);
    }

    // Ordena os vetores de destino para garantir determinismo
    for inner_map in adjacency.values_mut() {
        for targets in inner_map.values_mut() {
            targets.sort();
        }
    }

    YcgGraphOptimized {
        metadata: graph.metadata,
        definitions: graph.definitions,
        adjacency,
    }
}

fn convert_scip_to_ycg(index: scip_proto::Index, config: &YcgConfig) -> YcgGraph {
    let mut symbol_kind_map: HashMap<String, i32> = HashMap::new();
    let enricher = TreeSitterEnricher::new();

    for info in &index.external_symbols {
        symbol_kind_map.insert(info.symbol.clone(), info.kind);
    }
    for doc in &index.documents {
        for info in &doc.symbols {
            symbol_kind_map.insert(info.symbol.clone(), info.kind);
        }
    }

    convert_with_two_passes(index, symbol_kind_map, enricher, config)
}

fn convert_with_two_passes(
    index: scip_proto::Index,
    kind_map: HashMap<String, i32>,
    mut enricher: TreeSitterEnricher,
    config: &YcgConfig,
) -> YcgGraph {
    let mut nodes: Vec<SymbolNode> = Vec::new();
    let mut edges_set: HashSet<ReferenceEdge> = HashSet::new();
    let mut registry: HashMap<u64, String> = HashMap::new();
    let project_root = &config.project_root;

    // --- PASSADA A ---
    for doc in &index.documents {
        let file_id = xxh64(doc.relative_path.as_bytes(), 0);
        let file_anchor = generate_anchor("file", file_id);
        registry.insert(file_id, file_anchor);

        for occurrence in &doc.occurrences {
            if (occurrence.symbol_roles & scip_proto::SymbolRole::Definition as i32) != 0 {
                let clean_name = extract_name_from_uri(&occurrence.symbol);
                let id = xxh64(occurrence.symbol.as_bytes(), 0);
                let base = if clean_name.is_empty()
                    || clean_name.ends_with(".ts")
                    || clean_name.ends_with(".rs")
                {
                    "def".to_string()
                } else {
                    clean_name.replace(|c: char| !c.is_alphanumeric(), "_")
                };
                let anchor = generate_anchor(&base, id);
                registry.insert(id, anchor);
            }
        }
    }

    // --- PASSADA B ---
    for doc in index.documents {
        let real_path = project_root.join(&doc.relative_path);
        let file_id = xxh64(doc.relative_path.as_bytes(), 0);
        let mut local_scopes = Vec::new();
        local_scopes.push(Scope {
            id: file_id,
            start_line: 0,
            end_line: 100000,
        });

        // B.1 DEFINIÇÕES
        for occurrence in &doc.occurrences {
            let is_def = (occurrence.symbol_roles & scip_proto::SymbolRole::Definition as i32) != 0;
            if is_def {
                let id = xxh64(occurrence.symbol.as_bytes(), 0);
                let raw_kind = kind_map.get(&occurrence.symbol).copied().unwrap_or(0);
                let kind = if raw_kind == 0 {
                    infer_kind_from_uri(&occurrence.symbol)
                } else {
                    map_kind(raw_kind)
                };

                let should_skip = match config.lod {
                    LevelOfDetail::Low => matches!(
                        kind,
                        ScipSymbolKind::Variable
                            | ScipSymbolKind::Interface
                            | ScipSymbolKind::Module
                    ),
                    LevelOfDetail::Medium => {
                        let is_local_var = kind == ScipSymbolKind::Variable
                            && !occurrence.symbol.contains('#')
                            && !occurrence.symbol.contains('.');
                        let is_param = occurrence.symbol.contains("().(");
                        is_local_var || is_param
                    }
                    LevelOfDetail::High => false,
                };
                if should_skip {
                    continue;
                }

                let clean_name = extract_name_from_uri(&occurrence.symbol);

                // Resolve generic variable names from source code
                // **Validates: Requirements 6.1, 6.3, 6.4, 6.5**
                let final_name = if kind == ScipSymbolKind::Variable && is_generic_name(&clean_name)
                {
                    let start_line = occurrence.range.get(0).copied().unwrap_or(0);
                    let start_col = occurrence.range.get(1).copied().unwrap_or(0);

                    match enricher.resolve_variable_name(
                        &real_path,
                        start_line as usize,
                        start_col as usize,
                    ) {
                        Some(resolved) => {
                            eprintln!(
                                "✓ Resolved generic name '{}' to '{}' at {}:{}",
                                clean_name,
                                resolved,
                                real_path.display(),
                                start_line
                            );
                            resolved
                        }
                        None => {
                            eprintln!(
                                "⚠️  Failed to resolve generic name '{}' at {}:{}, using SCIP name",
                                clean_name,
                                real_path.display(),
                                start_line
                            );
                            clean_name
                        }
                    }
                } else {
                    clean_name
                };

                let extracted_parent = extract_parent_id(&occurrence.symbol);

                let parent_anchor = match extracted_parent {
                    Some(pid) => {
                        if let Some(anchor) = registry.get(&pid) {
                            Some(anchor.clone())
                        } else {
                            registry.get(&file_id).cloned()
                        }
                    }
                    None => {
                        if kind == ScipSymbolKind::File {
                            None
                        } else {
                            registry.get(&file_id).cloned()
                        }
                    }
                };

                let start_line = occurrence.range.get(0).copied().unwrap_or(0);

                let (sig, doc, logic) =
                    if kind != ScipSymbolKind::File && kind != ScipSymbolKind::Module {
                        match enricher.enrich(&real_path, start_line as usize) {
                            Some(res) => {
                                // Only attach logic metadata to methods and functions
                                // Variables and other symbol kinds should not have logic metadata
                                // **Validates: Requirements 4.1, 4.2, 4.4**
                                let l = if matches!(
                                    kind,
                                    ScipSymbolKind::Method | ScipSymbolKind::Function
                                ) && !res.preconditions.is_empty()
                                {
                                    Some(LogicMetadata {
                                        preconditions: res.preconditions,
                                    })
                                } else {
                                    None
                                };

                                // Validate variable signatures to prevent inheriting method signatures
                                let validated_sig = if kind == ScipSymbolKind::Variable {
                                    validate_variable_signature(res.signature, &occurrence.symbol)
                                } else {
                                    res.signature
                                };

                                (validated_sig, res.documentation, l)
                            }
                            None => (None, None, None),
                        }
                    } else {
                        (None, None, None)
                    };

                let my_anchor = registry
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| generate_anchor("gen", id));

                nodes.push(SymbolNode {
                    id: my_anchor,
                    name: final_name,
                    kind,
                    parent_id: parent_anchor,
                    documentation: doc,
                    signature: sig,
                    logic,
                });

                if matches!(
                    kind,
                    ScipSymbolKind::Function | ScipSymbolKind::Method | ScipSymbolKind::Class
                ) {
                    let end_line = occurrence.range.get(2).copied().unwrap_or(start_line);
                    local_scopes.push(Scope {
                        id,
                        start_line,
                        end_line,
                    });
                }
            }
        }

        // B.2 REFERÊNCIAS
        for occurrence in &doc.occurrences {
            let is_def = (occurrence.symbol_roles & scip_proto::SymbolRole::Definition as i32) != 0;
            if !is_def {
                let ref_line = occurrence.range.get(0).copied().unwrap_or(0);
                let source_u64 = find_enclosing_scope(&local_scopes, ref_line).unwrap_or(file_id);
                let target_u64 = xxh64(occurrence.symbol.as_bytes(), 0);

                if source_u64 != target_u64 {
                    let from_anchor = registry
                        .get(&source_u64)
                        .cloned()
                        .unwrap_or_else(|| generate_anchor("ctx", source_u64));
                    let target_exists = registry.contains_key(&target_u64);
                    if !target_exists && !matches!(config.lod, LevelOfDetail::High) {
                        continue;
                    }
                    let to_anchor = registry
                        .get(&target_u64)
                        .cloned()
                        .unwrap_or_else(|| generate_anchor("ext", target_u64));

                    edges_set.insert(ReferenceEdge {
                        from: from_anchor,
                        to: to_anchor,
                        edge_type: EdgeType::Calls,
                    });
                }
            }
        }
    }

    let mut edges: Vec<ReferenceEdge> = edges_set.into_iter().collect();
    edges.sort();

    YcgGraph {
        metadata: ProjectMetadata {
            name: "ycg-v1.3".to_string(),
            version: "1.3.0".to_string(),
        },
        definitions: nodes,
        references: edges,
    }
}

// --- HELPERS (Inalterados) ---
fn generate_anchor(name: &str, id: u64) -> String {
    let suffix = format!("{:x}", id);
    let short_suffix = &suffix[0..4.min(suffix.len())];
    format!("{}_{}", name, short_suffix)
}
fn find_enclosing_scope(scopes: &[Scope], line: i32) -> Option<u64> {
    let mut best_scope: Option<u64> = None;
    let mut min_size = i32::MAX;
    for scope in scopes {
        if line >= scope.start_line && line <= scope.end_line {
            let size = scope.end_line - scope.start_line;
            if size < min_size {
                min_size = size;
                best_scope = Some(scope.id);
            }
        }
    }
    best_scope
}

/// Checks if a variable name is a generic SCIP-generated name.
///
/// SCIP generates generic names for local variables in the format: `[a-z]+[0-9]+`
/// Examples: status0, timestamp1, user2, result3
///
/// **Validates: Requirement 6.2**
fn is_generic_name(name: &str) -> bool {
    // Pattern: lowercase letters followed by digits
    // Examples: status0, timestamp1, user2
    if name.is_empty() {
        return false;
    }

    let mut has_letters = false;
    let mut has_digits = false;
    let mut seen_digit = false;

    for ch in name.chars() {
        if ch.is_ascii_lowercase() {
            if seen_digit {
                // Letter after digit - not a generic name
                return false;
            }
            has_letters = true;
        } else if ch.is_ascii_digit() {
            has_digits = true;
            seen_digit = true;
        } else {
            // Non-alphanumeric character - not a generic name
            return false;
        }
    }

    // Must have both letters and digits, and end with digits
    has_letters && has_digits && seen_digit
}

/// Validates that a variable signature is not a method signature.
///
/// Variables should have simple type signatures like `variableName: Type`,
/// not method signatures containing function patterns. This function implements
/// comprehensive validation to prevent variables from inheriting parent method signatures.
///
/// # Validation Patterns
/// Rejects signatures containing:
/// - `function` keyword (method declarations)
/// - `=>` arrow function syntax
/// - `async` keyword (async method pattern)
/// - `@Decorator(...)` patterns (method parameter decorators)
/// - Return type annotations after closing paren `) :`
/// - Empty parentheses `()` (method calls or declarations)
///
/// # Arguments
/// * `sig` - The signature to validate
/// * `symbol_uri` - The SCIP symbol URI for logging purposes
///
/// # Returns
/// * `None` if the signature contains method-like patterns (rejected)
/// * The original signature if it's valid for a variable
///
/// # Examples
/// ```
/// // ✅ Valid variable signatures (accepted):
/// // "userId: number"
/// // "username: string"
/// // "user: User"
///
/// // ❌ Invalid variable signatures (rejected):
/// // "findOne(@Param('id', ParseIntPipe) id:num):Promise<UserDto>"
/// // "async function getData(): Promise<Data>"
/// // "catch(exception: unknown, host: ArgumentsHost)"
/// ```
fn validate_variable_signature(sig: Option<String>, symbol_uri: &str) -> Option<String> {
    if let Some(ref s) = sig {
        // Pattern 1: Contains function keyword
        if s.contains("function") {
            eprintln!(
                "⚠️  Rejecting method signature for variable {}: contains 'function'",
                symbol_uri
            );
            return None;
        }

        // Pattern 2: Contains arrow function syntax
        if s.contains("=>") {
            eprintln!(
                "⚠️  Rejecting method signature for variable {}: contains '=>'",
                symbol_uri
            );
            return None;
        }

        // Pattern 3: Starts with async (method pattern)
        if s.trim().starts_with("async ") {
            eprintln!(
                "⚠️  Rejecting method signature for variable {}: starts with 'async'",
                symbol_uri
            );
            return None;
        }

        // Pattern 4: Contains parameter list with decorators (e.g., @Param('id', ParseIntPipe))
        if s.contains('@') && s.contains('(') {
            eprintln!(
                "⚠️  Rejecting method signature for variable {}: contains decorator pattern",
                symbol_uri
            );
            return None;
        }

        // Pattern 5: Contains return type annotation with colon after closing paren
        // Example: "findOne(id: number): Promise<UserDto>"
        if let Some(paren_pos) = s.rfind(')') {
            let after_paren = &s[paren_pos + 1..].trim();
            if after_paren.starts_with(':') {
                eprintln!(
                    "⚠️  Rejecting method signature for variable {}: contains return type annotation",
                    symbol_uri
                );
                return None;
            }
        }

        // Pattern 6: Contains empty parentheses (method call or declaration)
        if s.contains("()") {
            eprintln!(
                "⚠️  Rejecting method signature for variable {}: contains '()'",
                symbol_uri
            );
            return None;
        }
    }
    sig
}
fn extract_parent_id(symbol: &str) -> Option<u64> {
    let mut chars: Vec<char> = symbol.chars().collect();
    if let Some(&last) = chars.last() {
        if last == '.' {
            chars.pop();
        }
    }
    let mut idx = chars.len();
    while idx > 0 {
        idx -= 1;
        let c = chars[idx];
        if c == '#' || c == '.' || c == '/' || c == '`' {
            let end_idx = if c == '#' { idx + 1 } else { idx };
            let parent_str: String = chars[0..end_idx].iter().collect();
            if parent_str.len() < symbol.len() && !parent_str.is_empty() {
                return Some(xxhash_rust::xxh64::xxh64(parent_str.as_bytes(), 0));
            }
            break;
        }
    }
    None
}
fn map_kind(k: i32) -> ScipSymbolKind {
    use scip_proto::symbol_information::Kind;
    match Kind::try_from(k).unwrap_or(Kind::UnspecifiedKind) {
        Kind::Class => ScipSymbolKind::Class,
        Kind::Method => ScipSymbolKind::Method,
        Kind::Function => ScipSymbolKind::Function,
        Kind::Variable => ScipSymbolKind::Variable,
        Kind::Interface => ScipSymbolKind::Interface,
        Kind::Module => ScipSymbolKind::Module,
        _ => ScipSymbolKind::Variable,
    }
}
fn infer_kind_from_uri(uri: &str) -> ScipSymbolKind {
    if uri.ends_with("().") || uri.contains("#<constructor>") {
        return ScipSymbolKind::Method;
    }
    if uri.ends_with('#') {
        return ScipSymbolKind::Class;
    }
    if uri.contains("/match") || uri.contains("function") {
        return ScipSymbolKind::Function;
    }
    if uri.ends_with('/') {
        return ScipSymbolKind::File;
    }
    ScipSymbolKind::Variable
}
fn extract_name_from_uri(uri: &str) -> String {
    let trimmed = uri
        .trim_end_matches('.')
        .trim_end_matches("()")
        .trim_end_matches('#')
        .trim_end_matches('/');
    let last_part = trimmed.split('/').last().unwrap_or(trimmed);
    let clean = last_part.replace('`', "");
    if clean == "<constructor>" {
        return "constructor".to_string();
    }
    if clean.is_empty() {
        return "unknown".to_string();
    }
    clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_generic_name_valid() {
        // Valid generic names: lowercase letters followed by digits
        assert!(is_generic_name("status0"));
        assert!(is_generic_name("timestamp1"));
        assert!(is_generic_name("user2"));
        assert!(is_generic_name("result3"));
        assert!(is_generic_name("data0"));
        assert!(is_generic_name("value1"));
        assert!(is_generic_name("item2"));
        assert!(is_generic_name("a0"));
        assert!(is_generic_name("abc123"));
    }

    #[test]
    fn test_is_generic_name_invalid() {
        // Invalid: no digits
        assert!(!is_generic_name("status"));
        assert!(!is_generic_name("user"));
        assert!(!is_generic_name("data"));

        // Invalid: no letters
        assert!(!is_generic_name("0"));
        assert!(!is_generic_name("123"));

        // Invalid: starts with uppercase
        assert!(!is_generic_name("Status0"));
        assert!(!is_generic_name("User1"));

        // Invalid: starts with digit
        assert!(!is_generic_name("0user"));
        assert!(!is_generic_name("1status"));

        // Invalid: contains special characters
        assert!(!is_generic_name("user_id"));
        assert!(!is_generic_name("user-name"));
        assert!(!is_generic_name("user.name"));
        assert!(!is_generic_name("user$0"));

        // Invalid: letters after digits
        assert!(!is_generic_name("user0name"));
        assert!(!is_generic_name("status1data"));

        // Invalid: empty string
        assert!(!is_generic_name(""));

        // Invalid: real variable names
        assert!(!is_generic_name("userId"));
        assert!(!is_generic_name("userName"));
        assert!(!is_generic_name("activeUsers"));
        assert!(!is_generic_name("myVariable"));
    }

    #[test]
    fn test_is_generic_name_edge_cases() {
        // Edge case: single letter + single digit
        assert!(is_generic_name("a0"));
        assert!(is_generic_name("z9"));

        // Edge case: many letters + many digits
        assert!(is_generic_name("abcdefghij0123456789"));

        // Edge case: mixed case (should fail)
        assert!(!is_generic_name("User0"));
        assert!(!is_generic_name("uSer0"));

        // Edge case: only letters (should fail)
        assert!(!is_generic_name("abcdef"));

        // Edge case: only digits (should fail)
        assert!(!is_generic_name("123456"));
    }
}
