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
        let signature = if let Some(idx) = raw_text.find('{') {
            Some(raw_text[..idx].trim().to_string())
        } else {
            Some(raw_text.trim().to_string())
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
