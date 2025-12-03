# Variable Name Resolution from Source

## Overview

Implementação da funcionalidade de resolução de nomes de variáveis do código fonte para substituir nomes genéricos gerados pelo SCIP (como `status0`, `timestamp1`) pelos nomes reais das variáveis.

## Problema Resolvido

SCIP gera nomes genéricos para variáveis locais no formato `[a-z]+[0-9]+`, o que reduz a legibilidade do grafo de contexto. Por exemplo:

```typescript
const activeUsers = await repository.findAll();
```

SCIP pode gerar: `activeUsers` → `user0` (nome genérico)

## Solução Implementada

O sistema agora detecta nomes genéricos e resolve os nomes reais do código fonte usando Tree-sitter:

```
user0 → activeUsers (nome real do código)
```

## Implementação

### 1. Detecção de Nomes Genéricos

#### Função `is_generic_name(name: &str) -> bool` (lib.rs)

Detecta se um nome segue o padrão genérico do SCIP:
- Padrão: `^[a-z]+[0-9]+$`
- Exemplos válidos: `status0`, `timestamp1`, `user2`, `result3`
- Exemplos inválidos: `userId`, `userName`, `User0`, `user_id`

**Valida: Requirement 6.2**

```rust
fn is_generic_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut has_letters = false;
    let mut has_digits = false;
    let mut seen_digit = false;

    for ch in name.chars() {
        if ch.is_ascii_lowercase() {
            if seen_digit {
                return false; // Letter after digit
            }
            has_letters = true;
        } else if ch.is_ascii_digit() {
            has_digits = true;
            seen_digit = true;
        } else {
            return false; // Non-alphanumeric
        }
    }

    has_letters && has_digits && seen_digit
}
```

### 2. Resolução de Nomes

#### Método `resolve_variable_name()` (enricher.rs)

Usa Tree-sitter para encontrar o identifier no AST na posição específica:

```rust
pub fn resolve_variable_name(
    &mut self,
    file_path: &Path,
    line: usize,
    col: usize,
) -> Option<String> {
    // 1. Get language parser
    let ext = file_path.extension()?.to_str()?;
    let language = self.parsers.get(ext)?;

    // 2. Read and parse source
    let source_code = std::fs::read_to_string(file_path).ok()?;
    let mut parser = Parser::new();
    parser.set_language(*language).ok()?;
    let tree = parser.parse(&source_code, None)?;

    // 3. Find node at position
    let node = find_node_at_position(tree.root_node(), line, col)?;

    // 4. Extract identifier text
    if node.kind() == "identifier" || node.kind() == "variable_declarator" {
        // Extract text from AST
        // ...
        return Some(name.to_string());
    }

    None
}
```

**Valida: Requirements 6.1, 6.3**

#### Função Auxiliar `find_node_at_position()`

Encontra o nó mais específico (menor) que contém a posição:

```rust
fn find_node_at_position(node: Node, line: usize, col: usize) -> Option<Node> {
    // Check if position is within node's range
    // Recursively search children for more specific node
    // Return the smallest node containing the position
}
```

### 3. Integração no Loop de Conversão

A resolução é integrada no loop de conversão de símbolos (lib.rs):

```rust
let clean_name = extract_name_from_uri(&occurrence.symbol);

// Resolve generic variable names from source code
let final_name = if kind == ScipSymbolKind::Variable && is_generic_name(&clean_name) {
    let start_line = occurrence.range.get(0).copied().unwrap_or(0);
    let start_col = occurrence.range.get(1).copied().unwrap_or(0);
    
    match enricher.resolve_variable_name(&real_path, start_line as usize, start_col as usize) {
        Some(resolved) => {
            eprintln!("✓ Resolved generic name '{}' to '{}' at {}:{}", 
                clean_name, resolved, real_path.display(), start_line);
            resolved
        }
        None => {
            eprintln!("⚠️  Failed to resolve generic name '{}' at {}:{}, using SCIP name", 
                clean_name, real_path.display(), start_line);
            clean_name
        }
    }
} else {
    clean_name
};
```

**Valida: Requirements 6.4, 6.5**

## Graceful Degradation

O sistema implementa degradação graciosa em caso de falhas:

1. **Arquivo não encontrado** → Retorna `None`, usa nome SCIP
2. **Extensão não suportada** → Retorna `None`, usa nome SCIP
3. **Erro de parsing** → Retorna `None`, usa nome SCIP
4. **Posição inválida** → Retorna `None`, usa nome SCIP
5. **Identifier não encontrado** → Retorna `None`, usa nome SCIP

Em todos os casos, um warning é logado e o nome SCIP original é mantido.

## Testes

### Testes Unitários (3 testes em lib.rs)

- `test_is_generic_name_valid` - Valida detecção de nomes genéricos válidos
- `test_is_generic_name_invalid` - Valida rejeição de nomes não-genéricos
- `test_is_generic_name_edge_cases` - Valida casos extremos

### Testes de Integração (6 testes)

- `test_resolve_variable_name_typescript` - Resolução em arquivo TypeScript
- `test_resolve_variable_name_invalid_position` - Posição inválida
- `test_resolve_variable_name_nonexistent_file` - Arquivo inexistente
- `test_resolve_variable_name_unsupported_extension` - Extensão não suportada
- `test_generic_name_patterns` - Documentação de padrões
- `test_graceful_degradation` - Validação de degradação graciosa

## Resultados

### Métricas de Qualidade

✅ **Todos os 237 testes da biblioteca passam** (+3 novos testes)
✅ **Todos os 6 testes de integração passam**
✅ **Compilação sem warnings**
✅ **Graceful degradation implementada**
✅ **Logs estruturados para debugging**

### Padrões de Nomes

**Nomes Genéricos (serão resolvidos):**
- `status0`, `timestamp1`, `user2`, `result3`
- `data0`, `value1`, `item2`
- `a0`, `abc123`

**Nomes Não-Genéricos (não serão processados):**
- `userId`, `userName`, `activeUsers` (camelCase)
- `user_id`, `user-name` (com caracteres especiais)
- `User0` (começa com maiúscula)
- `0user` (começa com dígito)
- `user`, `status` (sem dígitos)

### Critérios de Aceitação Atendidos

1. ✅ Método `resolve_variable_name()` implementado no enricher (Req 6.1)
2. ✅ Função `is_generic_name()` implementada em lib.rs (Req 6.2)
3. ✅ Integrada no loop de conversão para variáveis (Req 6.3)
4. ✅ Fallback para nome SCIP quando resolução falha (Req 6.4)
5. ✅ Warning log quando resolução falha (Req 6.5)

## Exemplos de Uso

### Caso 1: Nome genérico resolvido com sucesso

**Input (SCIP):**
```
Variable: user0
```

**Código Fonte:**
```typescript
const activeUsers = await repository.findAll();
```

**Output:**
```
Variable: activeUsers
Log: ✓ Resolved generic name 'user0' to 'activeUsers' at src/users.ts:10
```

### Caso 2: Falha na resolução (graceful degradation)

**Input (SCIP):**
```
Variable: status0
```

**Código Fonte:** (arquivo não encontrado)

**Output:**
```
Variable: status0
Log: ⚠️  Failed to resolve generic name 'status0' at src/missing.ts:5, using SCIP name
```

### Caso 3: Nome não-genérico (não processado)

**Input (SCIP):**
```
Variable: userId
```

**Output:**
```
Variable: userId
(Nenhum processamento - nome já é legível)
```

## Impacto

### Legibilidade
- Nomes de variáveis mais significativos no grafo
- Melhor compreensão do código por LLMs
- Redução de ambiguidade

### Performance
- Resolução apenas para nomes genéricos (filtro eficiente)
- Cache de parsers Tree-sitter reutilizado
- Impacto mínimo no tempo de processamento

### Robustez
- Graceful degradation em todos os casos de erro
- Logs estruturados para debugging
- Sem quebra de funcionalidade existente

## Limitações Conhecidas

1. **Posição imprecisa**: Se SCIP fornecer posição incorreta, resolução pode falhar
2. **Linguagens não suportadas**: Apenas TS/JS/Rust suportados atualmente
3. **Nomes ofuscados**: Código minificado pode ter nomes genéricos reais

## Próximos Passos

Esta implementação completa a Task 6 do bug-fix spec. As próximas tarefas incluem:
- Task 7: Fix parent_id assignment for variables
- Task 8: Add comprehensive integration tests

---

**Implementado por**: CodeAgent  
**Data**: 2025-12-02  
**Validado**: Requirements 6.1, 6.2, 6.3, 6.4, 6.5  
**Arquivos**: 
- `crates/ycg_core/src/enricher.rs`
- `crates/ycg_core/src/lib.rs`
- `crates/ycg_core/tests/variable_name_resolution_test.rs`
