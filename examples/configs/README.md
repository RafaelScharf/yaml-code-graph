# YCG Configuration Examples

Este diretório contém exemplos de configuração para diferentes casos de uso do YCG (YAML Code Graph).

## 📋 Índice de Configurações

| Configuração | Caso de Uso | Formato | Tokens | Detalhes |
|--------------|-------------|---------|--------|----------|
| [minimal.json](#minimal) | Teste rápido | YAML | 100% | Configuração mínima |
| [typescript-standard.json](#typescript-standard) | Projetos TypeScript | YAML | ~50% | Compacto + Framework Noise |
| [llm-optimized.json](#llm-optimized) | LLMs (GPT, Claude) | Ad-Hoc | ~20% | Máxima compressão |
| [adhoc-default.json](#adhoc-default) | Formato Ad-Hoc básico | Ad-Hoc | ~70% | Sem enriquecimento |
| [adhoc-signatures.json](#adhoc-signatures) | Com assinaturas | Ad-Hoc | ~80% | Level 1 |
| [adhoc-logic.json](#adhoc-logic) | Com lógica inline | Ad-Hoc | ~90% | Level 2 |
| [documentation.json](#documentation) | Documentação | Ad-Hoc | ~60% | Assinaturas + Noise Filter |
| [architecture-analysis.json](#architecture-analysis) | Análise arquitetural | YAML | ~40% | Apenas estrutura |

---

## 🚀 Como Usar

### Método 1: Copiar para o projeto

```bash
# Copie a configuração desejada para a raiz do seu projeto
cp examples/configs/llm-optimized.json ./ycg.config.json

# Execute o YCG (ele detectará automaticamente o config)
ycg generate --input index.scip --output graph.yaml
```

### Método 2: Especificar caminho (futuro)

```bash
# Nota: Esta funcionalidade será implementada em versões futuras
ycg generate --input index.scip --config examples/configs/llm-optimized.json
```

### Método 3: Usar flags CLI (sobrescreve config)

```bash
# As flags CLI têm precedência sobre o arquivo de configuração
ycg generate --input index.scip --output-format adhoc --compact
```

## 🧪 Testar Configurações

Você pode testar todas as configurações nos projetos exemplo:

```bash
# Da raiz do projeto
make test-configs

# Ou diretamente
./examples/configs/test-configs.sh
```

**O que o teste faz**:
1. ✅ Verifica se o binário YCG está compilado
2. ✅ Gera índices SCIP para os projetos exemplo (se necessário)
3. ✅ Testa cada configuração em `simple-ts` e `nestjs-api-ts`
4. ✅ Gera outputs em `examples/configs/outputs/`
5. ✅ Cria relatório com métricas (linhas, tamanho, tokens, tempo)

**Outputs gerados**:
```
examples/configs/outputs/
├── simple-ts/
│   ├── minimal.yaml
│   ├── typescript-standard.yaml
│   ├── adhoc-default.yaml
│   ├── adhoc-signatures.yaml
│   ├── adhoc-logic.yaml
│   ├── llm-optimized.yaml
│   ├── documentation.yaml
│   └── architecture-analysis.yaml
├── nestjs-api-ts/
│   └── (mesmos arquivos)
├── metrics.csv
└── README.md
```

**Ver resultados**:
```bash
# Ver métricas
cat examples/configs/outputs/metrics.csv

# Comparar tamanhos
ls -lh examples/configs/outputs/simple-ts/

# Comparar dois configs
diff examples/configs/outputs/simple-ts/minimal.yaml \
     examples/configs/outputs/simple-ts/llm-optimized.yaml
```

**Limpar outputs**:
```bash
make clean-configs
```

---

## 📖 Detalhes das Configurações

### <a name="minimal"></a>1. minimal.json

**Caso de uso**: Teste rápido, configuração padrão

```bash
cp examples/configs/minimal.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato YAML padrão
- ❌ Sem compactação
- ❌ Sem filtros
- 📊 Tokens: 100% (baseline)

**Quando usar**: Primeira vez usando YCG, debugging, comparação de outputs

---

### <a name="typescript-standard"></a>2. typescript-standard.json

**Caso de uso**: Projetos TypeScript/JavaScript em produção

```bash
cp examples/configs/typescript-standard.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato YAML
- ✅ Compactação ativada (remove símbolos locais)
- ✅ Framework Noise Filter (remove decorators, DI constructors)
- ✅ Exclui testes e node_modules
- ✅ Inclui apenas `src/**/*.ts` e `src/**/*.tsx`
- 📊 Tokens: ~50% do baseline

**Quando usar**: 
- Projetos NestJS, Angular, React
- Análise de código TypeScript
- Documentação de APIs

**Exemplo de output**:
```yaml
_meta:
  name: ycg-v1.3
  version: 1.3.0
_defs:
  - id: UserService_a1b2
    n: UserService
    t: class
  - id: createUser_c3d4
    n: createUser
    t: method
graph:
  UserService_a1b2:
    calls:
      - Database_x1y2
```

---

### <a name="llm-optimized"></a>4. llm-optimized.json ⭐ RECOMENDADO PARA LLMs

**Caso de uso**: Máxima compressão para contexto de LLMs (GPT-4, Claude, etc)

```bash
cp examples/configs/llm-optimized.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato Ad-Hoc (pipe-separated)
- ✅ Compactação ativada
- ✅ Framework Noise Filter ativado
- ✅ Granularidade: **Logic** (Level 2)
- ✅ Exclui testes, node_modules, target, dist
- 📊 Tokens: ~20% do baseline (80% de redução!)

**Quando usar**:
- Enviar código para LLMs
- Contexto limitado (< 100k tokens)
- Análise de lógica de negócio

**Exemplo de output**:
```yaml
_defs:
  - "UserService_a1b2|class UserService|class|logic:"
  - "createUser_c3d4|async createUser(name: string): Promise<User>|method|logic:validate_input,create_instance,save_to_db,return_result"
graph:
  UserService_a1b2:
    calls:
      - Database_x1y2
```

**Comparação de tokens**:
```
Baseline (minimal):     10,000 tokens
typescript-standard:     5,000 tokens (50%)
llm-optimized:           2,000 tokens (20%) ✨
```

---

### <a name="adhoc-default"></a>5. adhoc-default.json

**Caso de uso**: Formato Ad-Hoc sem enriquecimento (Level 0)

```bash
cp examples/configs/adhoc-default.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato Ad-Hoc
- ❌ Sem compactação
- ❌ Sem Framework Noise Filter
- ✅ Granularidade: **Default** (apenas ID|Name|Type)
- 📊 Tokens: ~70% do baseline

**Quando usar**:
- Comparar formato Ad-Hoc vs YAML
- Baseline para granularidade

**Exemplo de output**:
```yaml
_defs:
  - "UserService_a1b2|UserService|class"
  - "createUser_c3d4|createUser|method"
```

---

### <a name="adhoc-signatures"></a>6. adhoc-signatures.json

**Caso de uso**: Formato Ad-Hoc com assinaturas (Level 1)

```bash
cp examples/configs/adhoc-signatures.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato Ad-Hoc
- ❌ Sem compactação
- ❌ Sem Framework Noise Filter
- ✅ Granularidade: **Signatures** (ID|Signature|Type)
- 📊 Tokens: ~80% do baseline

**Quando usar**:
- Documentação de APIs
- Análise de interfaces
- Verificação de tipos

**Exemplo de output**:
```yaml
_defs:
  - "UserService_a1b2|class UserService|class"
  - "createUser_c3d4|async createUser(name: string): Promise<User>|method"
```

---

### <a name="adhoc-logic"></a>7. adhoc-logic.json

**Caso de uso**: Formato Ad-Hoc com lógica inline (Level 2)

```bash
cp examples/configs/adhoc-logic.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato Ad-Hoc
- ❌ Sem compactação
- ❌ Sem Framework Noise Filter
- ✅ Granularidade: **Logic** (ID|Signature|Type|Logic)
- 📊 Tokens: ~90% do baseline

**Quando usar**:
- Análise de lógica de negócio
- Compreensão de fluxos complexos
- Debugging

**Exemplo de output**:
```yaml
_defs:
  - "UserService_a1b2|class UserService|class|logic:"
  - "createUser_c3d4|async createUser(name: string): Promise<User>|method|logic:validate_input,create_instance,save_to_db,return_result"
```

---

### <a name="documentation"></a>8. documentation.json

**Caso de uso**: Geração de documentação técnica

```bash
cp examples/configs/documentation.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato Ad-Hoc
- ❌ Sem compactação (mantém todos os símbolos públicos)
- ✅ Framework Noise Filter (remove boilerplate)
- ✅ Granularidade: **Signatures**
- ✅ Exclui testes
- ✅ Inclui apenas `src/`
- 📊 Tokens: ~60% do baseline

**Quando usar**:
- Gerar documentação de APIs
- Onboarding de desenvolvedores
- Revisão de código

---

### <a name="architecture-analysis"></a>9. architecture-analysis.json

**Caso de uso**: Análise de arquitetura de alto nível

```bash
cp examples/configs/architecture-analysis.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
```

**Características**:
- ✅ Formato YAML
- ✅ Compactação ativada (apenas símbolos exportados)
- ✅ Framework Noise Filter
- ✅ Exclui testes, migrations, seeds
- ✅ Inclui apenas `src/`
- 📊 Tokens: ~40% do baseline

**Quando usar**:
- Visualizar dependências entre módulos
- Identificar acoplamento
- Refatoração de arquitetura
- Diagramas de alto nível

---

## 🎯 Guia de Decisão

### Escolha por Objetivo

```
┌─────────────────────────────────────────────────────────────┐
│ Qual é o seu objetivo?                                      │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
    Enviar para LLM    Documentação    Análise Arquitetura
        │                   │                   │
        ▼                   ▼                   ▼
  llm-optimized.json  documentation.json  architecture-analysis.json
```

### Escolha por Linguagem

```
TypeScript/JavaScript  →  typescript-standard.json
Múltiplas linguagens   →  llm-optimized.json
```

### Escolha por Tamanho do Projeto

```
Pequeno (< 10k LOC)    →  minimal.json ou typescript-standard.json
Médio (10k-50k LOC)    →  typescript-standard.json
Grande (> 50k LOC)     →  llm-optimized.json ou architecture-analysis.json
```

---

## 🔧 Customização

### Modificar uma configuração existente

1. Copie a configuração base:
```bash
cp examples/configs/llm-optimized.json ./ycg.config.json
```

2. Edite o arquivo:
```json
{
  "output": {
    "format": "adhoc",
    "compact": true,
    "adhocGranularity": "signatures"  // Mudou de "logic" para "signatures"
  }
}
```

3. Execute:
```bash
ycg generate --input index.scip --output graph.yaml
```

### Sobrescrever com flags CLI

As flags CLI sempre têm precedência sobre o arquivo de configuração:

```bash
# Config diz "format": "yaml", mas CLI sobrescreve para adhoc
ycg generate --input index.scip --output-format adhoc
```

---

## 📊 Comparação de Outputs

### Exemplo: UserService com 3 métodos

| Config | Símbolos | Linhas | Tokens | Tempo |
|--------|----------|--------|--------|-------|
| minimal | 15 | 200 | 10,000 | 1.2s |
| typescript-standard | 8 | 100 | 5,000 | 1.0s |
| adhoc-default | 15 | 150 | 7,000 | 1.1s |
| adhoc-signatures | 15 | 180 | 8,000 | 1.3s |
| adhoc-logic | 15 | 220 | 9,000 | 1.5s |
| llm-optimized | 3 | 50 | 2,000 | 0.8s |
| architecture-analysis | 5 | 70 | 3,500 | 0.9s |

---

## 🆘 Troubleshooting

### Config não está sendo carregado

```bash
# Verifique se o arquivo está na raiz do projeto
ls -la ycg.config.json

# Verifique se o JSON é válido
cat ycg.config.json | jq .
```

### Muitos tokens ainda

```bash
# Use a configuração mais agressiva
cp examples/configs/llm-optimized.json ./ycg.config.json

# Ou adicione mais exclusões
{
  "ignore": {
    "customPatterns": [
      "**/vendor/**",
      "**/third_party/**"
    ]
  }
}
```

### Faltando símbolos importantes

```bash
# Desative compactação
{
  "output": {
    "compact": false  // Mantém todos os símbolos
  }
}

# Ou use LOD mais alto via CLI
ycg generate --input index.scip --lod 2
```

---

## 📚 Referências

- [YCG Documentation](../../README.md)
- [Optimization Guide](../../OPTIMIZATION_GUIDE.md)
- [CLI Reference](../../CLI_REFERENCE.md)
- [Granularity Guide](../../GRANULARITY_GUIDE.md)

---

## 🛠️ Scripts Auxiliares

### test-configs.sh

Testa todas as configurações nos projetos exemplo:

```bash
./examples/configs/test-configs.sh
```

**Funcionalidades**:
- ✅ Compila o projeto automaticamente
- ✅ Gera índices SCIP se necessário
- ✅ Testa cada config em simple-ts e nestjs-api-ts
- ✅ Gera métricas detalhadas
- ✅ Cria relatório comparativo

### compare-outputs.sh

Compara os outputs gerados:

```bash
./examples/configs/compare-outputs.sh
```

**Mostra**:
- 📊 Tabela comparativa de linhas/tamanho/tokens
- 📉 Redução percentual vs baseline
- 🎯 Recomendações por caso de uso

## 🤝 Contribuindo

Tem uma configuração útil? Abra um PR!

```bash
# 1. Crie sua configuração
vim examples/configs/minha-config.json

# 2. Teste com os projetos exemplo
make test-configs

# 3. Verifique os resultados
make compare-configs

# 4. Adicione documentação neste README
# 5. Abra um PR
```
