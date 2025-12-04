// crates/ycg_core/src/enricher.rs
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

pub struct TreeSitterEnricher {
    parsers: HashMap<String, Language>,
}

pub struct EnrichmentResult {
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub preconditions: Vec<String>, // Novo campo
}

impl TreeSitterEnricher {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        parsers.insert("rs".into(), tree_sitter_rust::language());
        parsers.insert("ts".into(), tree_sitter_typescript::language_typescript());
        parsers.insert("tsx".into(), tree_sitter_typescript::language_tsx());
        parsers.insert("js".into(), tree_sitter_javascript::language());
        Self { parsers }
    }

    /// Resolves a variable name from source code at a specific position.
    ///
    /// Uses Tree-sitter to find the identifier node at the given line and column,
    /// then extracts the actual variable name from the source code.
    ///
    /// # Arguments
    /// * `file_path` - Path to the source file
    /// * `line` - Zero-based line number
    /// * `col` - Zero-based column number
    ///
    /// # Returns
    /// * `Some(String)` - The resolved variable name from source
    /// * `None` - If resolution fails (file not found, parse error, no identifier at position)
    ///
    /// **Validates: Requirements 6.1, 6.3, 6.4, 6.5**
    pub fn resolve_variable_name(
        &mut self,
        file_path: &Path,
        line: usize,
        col: usize,
    ) -> Option<String> {
        // Get language parser for file extension
        let ext = file_path.extension()?.to_str()?;
        let language = self.parsers.get(ext)?;

        // Read source code
        let source_code = std::fs::read_to_string(file_path).ok()?;

        // Parse the file
        let mut parser = Parser::new();
        parser.set_language(*language).ok()?;
        let tree = parser.parse(&source_code, None)?;

        // Find node at the specified position
        let node = find_node_at_position(tree.root_node(), line, col)?;

        // Extract identifier text
        if node.kind() == "identifier" || node.kind() == "variable_declarator" {
            let text = &source_code[node.start_byte()..node.end_byte()];

            // For variable_declarator, we need to find the identifier child
            if node.kind() == "variable_declarator" {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "identifier" {
                        let name = &source_code[child.start_byte()..child.end_byte()];
                        return Some(name.to_string());
                    }
                }
            }

            return Some(text.to_string());
        }

        None
    }

    pub fn enrich(&mut self, file_path: &Path, start_line: usize) -> Option<EnrichmentResult> {
        let ext = file_path.extension()?.to_str()?;
        let language = self.parsers.get(ext)?;
        let source_code = std::fs::read_to_string(file_path).ok()?;

        let mut parser = Parser::new();
        parser.set_language(*language).ok()?;

        let tree = parser.parse(&source_code, None)?;
        let root = tree.root_node();

        let target_node = find_deepest_definition(root, start_line)?;

        // 1. Assinatura
        let raw_text = &source_code[target_node.start_byte()..target_node.end_byte()];
        let signature = if let Some(idx) = find_body_start(raw_text) {
            let sig = raw_text[..idx].trim().to_string();
            // Validate signature is not truncated
            if is_truncated(&sig) {
                eprintln!(
                    "Warning: Truncated signature at {}:{} - falling back to symbol name",
                    file_path.display(),
                    start_line
                );
                None
            } else {
                Some(sig)
            }
        } else {
            let sig = raw_text.trim().to_string();
            // Validate signature is not truncated
            if is_truncated(&sig) {
                eprintln!(
                    "Warning: Truncated signature at {}:{} - falling back to symbol name",
                    file_path.display(),
                    start_line
                );
                None
            } else {
                Some(sig)
            }
        };

        // 2. Documentação
        let documentation = extract_comments(target_node, &source_code);

        // 3. Logic Lifting (Extração de Pré-condições)
        let preconditions = extract_guard_clauses(target_node, &source_code, *language);

        Some(EnrichmentResult {
            signature,
            documentation,
            preconditions,
        })
    }
}

// ... (find_deepest_definition e extract_comments MANTIDOS IGUAIS - não apague) ...
// Copie as funções anteriores aqui se for substituir o arquivo todo.
// Vou adicionar apenas a nova função abaixo:

const DEFINITION_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "method_definition",
    "public_field_definition",
    "property_signature",
    "lexical_declaration",
    "variable_declaration",
    "function_item",
    "struct_item",
    "impl_item",
];

fn find_deepest_definition(node: Node, target_line: usize) -> Option<Node> {
    let start = node.start_position().row;
    let end = node.end_position().row;
    if target_line < start || target_line > end {
        return None;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(deepest) = find_deepest_definition(child, target_line) {
            return Some(deepest);
        }
    }
    if DEFINITION_KINDS.contains(&node.kind()) {
        return Some(node);
    }
    None
}

fn extract_comments(node: Node, source: &str) -> Option<String> {
    let mut comments = Vec::new();
    let mut cursor = node.prev_sibling();
    while let Some(sibling) = cursor {
        let kind = sibling.kind();
        if kind == "comment" || kind == "line_comment" || kind == "block_comment" {
            let text = &source[sibling.start_byte()..sibling.end_byte()];
            let clean = text
                .replace("///", "")
                .replace("/**", "")
                .replace("*/", "")
                .replace("*", "")
                .trim()
                .to_string();
            comments.push(clean);
            cursor = sibling.prev_sibling();
        } else {
            break;
        }
    }
    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join("\n"))
    }
}

// --- LÓGICA NOVA: Logic Lifter ---

fn extract_guard_clauses(node: Node, source: &str, lang: Language) -> Vec<String> {
    let mut preconditions = Vec::new();

    // Query para TypeScript/Rust: Procura IFs que tenham 'throw' ou 'return' dentro
    // Esta query é simplificada para demonstração
    let query_str = "
        (if_statement
            condition: (_) @cond
            consequence: (statement_block) @block
        )
    ";

    if let Ok(query) = Query::new(lang, query_str) {
        let mut cursor = QueryCursor::new();
        // Executa a query APENAS dentro do nó da função atual (não no arquivo todo)
        let matches = cursor.matches(&query, node, source.as_bytes());

        for m in matches {
            // Verifica se o bloco do IF tem um 'throw' ou 'return' (indicando guard clause)
            let block_node = m.captures[1].node; // captura @block
            let block_text = &source[block_node.start_byte()..block_node.end_byte()];

            if block_text.contains("throw") || block_text.contains("return") {
                // Captura a condição
                let cond_node = m.captures[0].node; // captura @cond
                let cond_text = &source[cond_node.start_byte()..cond_node.end_byte()];

                // Remove parenteses extras se houver
                let clean_cond = cond_text.trim_matches(|c| c == '(' || c == ')').trim();

                // Inverte a lógica (Human Readable): "Se x < 0 falha" vira "Requer x >= 0"
                // Para o MVP, vamos apenas retornar a condição crua prefixada
                preconditions.push(format!("must avoid: {}", clean_cond));
            }
        }
    }

    preconditions
}
/// Finds the start of the function body by matching brackets.
/// Returns the index of the first unmatched opening brace '{'.
/// Handles nested parentheses and angle brackets to avoid false positives.
fn find_body_start(text: &str) -> Option<usize> {
    let mut paren_depth = 0;
    let mut angle_depth = 0;

    for (i, ch) in text.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '<' => angle_depth += 1,
            '>' => angle_depth -= 1,
            '{' if paren_depth == 0 && angle_depth == 0 => return Some(i),
            _ => {}
        }
    }

    None
}

/// Detects if a signature is truncated or incomplete.
/// Checks for unmatched brackets and incomplete patterns.
fn is_truncated(signature: &str) -> bool {
    signature.ends_with('(')
        || signature.ends_with('<')
        || signature.matches('(').count() != signature.matches(')').count()
        || signature.matches('<').count() != signature.matches('>').count()
}

/// Finds the smallest node at a specific position in the AST.
///
/// Recursively searches the tree to find the most specific (deepest) node
/// that contains the given line and column position.
///
/// # Arguments
/// * `node` - The root node to start searching from
/// * `line` - Zero-based line number
/// * `col` - Zero-based column number
///
/// # Returns
/// * `Some(Node)` - The smallest node containing the position
/// * `None` - If no node contains the position
fn find_node_at_position(node: Node, line: usize, col: usize) -> Option<Node> {
    let start_pos = node.start_position();
    let end_pos = node.end_position();

    // Check if position is within this node's range
    if line < start_pos.row || line > end_pos.row {
        return None;
    }

    if line == start_pos.row && col < start_pos.column {
        return None;
    }

    if line == end_pos.row && col > end_pos.column {
        return None;
    }

    // Try to find a more specific child node
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = find_node_at_position(child, line, col) {
            return Some(found);
        }
    }

    // No more specific child found, return this node
    Some(node)
}
