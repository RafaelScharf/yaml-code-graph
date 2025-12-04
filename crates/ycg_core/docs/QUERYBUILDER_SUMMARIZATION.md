# QueryBuilder Signature Summarization

## Overview

Implementação da funcionalidade de sumarização de assinaturas QueryBuilder para reduzir drasticamente o tamanho de assinaturas de variáveis que contêm queries complexas.

## Problema Resolvido

Assinaturas de variáveis com QueryBuilder frequentemente contêm 30+ linhas de código de query, poluindo o output e desperdiçando tokens. Por exemplo:

```typescript
const activeUsers = await this.userRepository
  .createQueryBuilder('User', 'u')
  .select('u.id, u.name, u.email, u.createdAt, u.updatedAt')
  .where('u.active = :active', { active: true })
  .andWhere('u.deletedAt IS NULL')
  .orderBy('u.createdAt', 'DESC')
  .limit(100)
  .getMany()
```

## Solução Implementada

A assinatura acima é agora sumarizada para:

```
activeUsers: User[]
```

## Implementação

### Funções Adicionadas

#### 1. `is_query_builder_pattern(sig: &str) -> bool`

Detecta se uma assinatura corresponde ao padrão QueryBuilder:
- Verifica presença de 2+ keywords: `createQueryBuilder`, `select`, `where`, `getMany`, `getOne`, `leftJoin`
- Verifica se o comprimento > 100 caracteres

**Valida: Requirements 5.1, 5.2**

#### 2. `extract_entity_type(sig: &str) -> Option<String>`

Extrai o tipo da entidade do padrão `createQueryBuilder('EntityName', ...)`:
- Suporta aspas simples e duplas
- Lida com casos sem aspas
- Retorna `None` se não encontrar tipo válido

**Valida: Requirement 5.3**

#### 3. `summarize_query_builder(sig: &str, var_name: &str) -> String`

Formata a assinatura sumarizada:
- `variableName: EntityType` para `getOne()`
- `variableName: EntityType[]` para `getMany()`
- `variableName: QueryResult` ou `QueryResult[]` como fallback

**Valida: Requirements 5.4, 5.6**

### Integração

A detecção e sumarização foram integradas na função `compact_signature()`, executando antes do processamento normal de compactação:

```rust
fn compact_signature(sig: &str, method_name: &str) -> String {
    // Check if this is a QueryBuilder pattern and summarize if so
    if Self::is_query_builder_pattern(sig) {
        return Self::summarize_query_builder(sig, method_name);
    }
    
    // ... resto da lógica de compactação
}
```

## Testes

### Testes Unitários (52 testes no módulo signature_extractor)

- `test_is_query_builder_pattern_positive` - Detecta padrão QB válido
- `test_is_query_builder_pattern_negative_too_short` - Rejeita QB curto
- `test_is_query_builder_pattern_negative_one_keyword` - Rejeita com 1 keyword
- `test_extract_entity_type_simple` - Extrai tipo simples
- `test_extract_entity_type_with_quotes` - Lida com aspas
- `test_extract_entity_type_complex` - Extrai de chain complexo
- `test_extract_entity_type_not_found` - Retorna None quando não encontra
- `test_summarize_query_builder_get_many` - Formata com array
- `test_summarize_query_builder_get_one` - Formata sem array
- `test_summarize_query_builder_fallback` - Usa QueryResult como fallback
- `test_compact_signature_with_query_builder` - Integração completa
- `test_compact_signature_query_builder_length_requirement` - Valida < 100 chars
- `test_query_builder_with_left_join` - Detecta com leftJoin
- `test_non_query_builder_long_signature` - Não sumariza não-QB

### Testes de Integração (7 testes)

- `test_query_builder_summarization_integration` - Teste end-to-end completo
- `test_query_builder_get_one_summarization` - Valida getOne sem array
- `test_query_builder_with_joins` - Valida com múltiplos joins
- `test_query_builder_fallback_to_query_result` - Valida fallback
- `test_non_query_builder_not_summarized` - Valida não-sumarização
- `test_short_query_builder_not_summarized` - Valida requisito de comprimento
- `test_query_builder_with_complex_entity_name` - Valida nomes complexos

## Resultados

### Métricas de Qualidade

✅ **Todos os 234 testes da biblioteca passam**
✅ **Todos os 52 testes do módulo signature_extractor passam**
✅ **Todos os 7 testes de integração passam**
✅ **Compilação sem warnings**
✅ **Sem warnings do clippy no código novo**

### Redução de Tamanho

Exemplo real:
- **Antes**: 250+ caracteres (query completa)
- **Depois**: 20-30 caracteres (`variableName: EntityType[]`)
- **Redução**: ~90% de redução no tamanho

### Critérios de Aceitação Atendidos

1. ✅ Função `is_query_builder_pattern()` detecta padrões QB (Req 5.1, 5.2)
2. ✅ Função `extract_entity_type()` infere tipo da entidade (Req 5.3)
3. ✅ Função `summarize_query_builder()` formata output (Req 5.4)
4. ✅ Integrada em `extract_signature()` (Req 5.4)
5. ✅ Assinaturas QB sumarizadas para < 100 chars (Req 5.6)

## Exemplos de Uso

### Caso 1: getMany com entidade explícita

**Input:**
```typescript
const users = await this.userRepository
  .createQueryBuilder('User')
  .select('user.id')
  .where('user.active = :active', { active: true })
  .getMany()
```

**Output:**
```
users: User[]
```

### Caso 2: getOne com entidade explícita

**Input:**
```typescript
const user = await this.userRepository
  .createQueryBuilder('User')
  .where('user.id = :id', { id: userId })
  .getOne()
```

**Output:**
```
user: User
```

### Caso 3: Fallback quando entidade não é clara

**Input:**
```typescript
const results = await someRepository
  .createQueryBuilder()
  .select('field1')
  .getMany()
```

**Output:**
```
results: QueryResult[]
```

### Caso 4: Com joins complexos

**Input:**
```typescript
const usersWithProfiles = await this.userRepository
  .createQueryBuilder('User', 'u')
  .leftJoin('u.profile', 'profile')
  .leftJoin('u.posts', 'posts')
  .select('u.id, profile.bio, posts.title')
  .getMany()
```

**Output:**
```
usersWithProfiles: User[]
```

## Impacto

### Performance
- Redução significativa no tamanho do output YAML
- Menos tokens consumidos por LLMs
- Grafos de contexto mais concisos

### Qualidade
- Informação essencial preservada (tipo da entidade)
- Ruído de implementação removido
- Melhor legibilidade do output

### Manutenibilidade
- Código bem testado (59 testes totais)
- Documentação inline completa
- Fallbacks robustos para casos edge

## Próximos Passos

Esta implementação completa a Task 5 do bug-fix spec. As próximas tarefas incluem:
- Task 6: Implement decorator stripping
- Task 7: Add comprehensive integration tests
- Task 8: Update documentation

---

**Implementado por**: CodeAgent  
**Data**: 2025-12-02  
**Validado**: Requirements 5.1, 5.2, 5.3, 5.4, 5.6  
**Arquivo**: `crates/ycg_core/src/signature_extractor.rs`
