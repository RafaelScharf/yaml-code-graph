# YCG Examples

Este diretório contém exemplos de projetos e configurações para o YCG.

## 📁 Estrutura

```
examples/
├── configs/              ⭐ Configurações organizadas
│   ├── README.md        📖 Documentação completa
│   ├── minimal.json
│   ├── typescript-standard.json
│   ├── llm-optimized.json
│   ├── adhoc-default.json
│   ├── adhoc-signatures.json
│   ├── adhoc-logic.json
│   ├── documentation.json
│   └── architecture-analysis.json
│
├── nestjs-api-ts/       📦 Projeto exemplo NestJS
├── simple-ts/           📦 Projeto exemplo TypeScript simples
│
└── ycg.config.*.json    ⚠️  DEPRECATED (use configs/)
```

## 🚀 Quick Start

### 1. Escolha uma configuração

Veja todas as opções em [`configs/README.md`](configs/README.md)

**Recomendações rápidas**:
- **Para LLMs**: `configs/llm-optimized.json` (80% menos tokens)
- **Para TypeScript**: `configs/typescript-standard.json`
- **Para documentação**: `configs/documentation.json`

### 2. Copie para seu projeto

```bash
cp examples/configs/llm-optimized.json ./ycg.config.json
```

### 3. Execute o YCG

```bash
# Gere o índice SCIP
ycg index --directory . --output index.scip

# Gere o grafo (usa ycg.config.json automaticamente)
ycg generate --input index.scip --output graph.yaml
```

## 📦 Projetos Exemplo

### nestjs-api-ts

Projeto NestJS completo com:
- Controllers
- Services
- DTOs
- Guards
- Modules

**Como usar**:
```bash
cd examples/nestjs-api-ts
npm install
ycg index --directory . --output index.scip
ycg generate --input index.scip --output context_map.xml
```

### simple-ts

Projeto TypeScript simples para testes rápidos.

**Como usar**:
```bash
cd examples/simple-ts
ycg index --directory . --output index.scip
ycg generate --input index.scip --output test_output.yaml
```

## 📖 Documentação Completa

Para documentação detalhada de cada configuração, veja:

👉 **[configs/README.md](configs/README.md)**

Inclui:
- ✅ Descrição de cada configuração
- ✅ Casos de uso
- ✅ Exemplos de comandos
- ✅ Comparação de outputs
- ✅ Guia de decisão
- ✅ Troubleshooting

## ⚠️ Arquivos Deprecated

Os seguintes arquivos estão deprecated e serão removidos na v2.0.0:

- ❌ `ycg.config.full.json` → Use `configs/llm-optimized.json`
- ❌ `ycg.config.granularity-default.json` → Use `configs/adhoc-default.json`
- ❌ `ycg.config.granularity-logic.json` → Use `configs/adhoc-logic.json`
- ❌ `ycg.config.granularity-signatures.json` → Use `configs/adhoc-signatures.json`
- ❌ `ycg.config.minimal.json` → Use `configs/minimal.json`
- ❌ `ycg.config.typescript.json` → Use `configs/typescript-standard.json`

## 🎯 Casos de Uso Comuns

### Enviar código para GPT-4/Claude

```bash
cp examples/configs/llm-optimized.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml
# Resultado: 80% menos tokens!
```

### Documentar API TypeScript

```bash
cp examples/configs/documentation.json ./ycg.config.json
ycg generate --input index.scip --output api-docs.yaml
```

### Analisar arquitetura Rust

```bash
cp examples/configs/architecture-analysis.json ./ycg.config.json
ycg generate --input index.scip --lod 0 --output architecture.yaml
```

### Comparar formatos

```bash
# YAML padrão
cp examples/configs/minimal.json ./ycg.config.json
ycg generate --input index.scip --output graph-yaml.yaml

# Ad-Hoc compacto
cp examples/configs/adhoc-default.json ./ycg.config.json
ycg generate --input index.scip --output graph-adhoc.yaml

# Comparar tamanhos
wc -l graph-*.yaml
```

## 🆘 Troubleshooting

### Config não está sendo carregado

```bash
# Verifique se está na raiz do projeto
ls -la ycg.config.json

# Valide o JSON
cat ycg.config.json | jq .
```

### Output muito grande

Use a configuração mais agressiva:
```bash
cp examples/configs/llm-optimized.json ./ycg.config.json
```

### Faltando símbolos

Desative compactação:
```bash
# Edite ycg.config.json
{
  "output": {
    "compact": false
  }
}
```

## 📚 Mais Recursos

- [YCG Documentation](../README.md)
- [Optimization Guide](../OPTIMIZATION_GUIDE.md)
- [CLI Reference](../CLI_REFERENCE.md)
- [Granularity Guide](../GRANULARITY_GUIDE.md)

## 🤝 Contribuindo

Tem um exemplo útil? Abra um PR!

1. Crie seu exemplo em `examples/`
2. Adicione documentação
3. Teste com projetos reais
4. Abra um PR

---

**Última atualização**: Dezembro 2024
