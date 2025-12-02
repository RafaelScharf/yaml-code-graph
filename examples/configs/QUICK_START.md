# YCG Configs - Quick Reference

Referência rápida para escolher e testar configurações.

## 🎯 Escolha Rápida

```
Preciso de...                    → Use esta config
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Máxima compressão para LLM       → llm-optimized.json        (80% redução)
Documentação legível             → documentation.json         (40% redução)
Análise de arquitetura           → architecture-analysis.json (60% redução)
Projeto TypeScript padrão        → typescript-standard.json   (50% redução)
Teste rápido / baseline          → minimal.json               (0% redução)
Formato Ad-Hoc básico            → adhoc-default.json         (30% redução)
Ad-Hoc com assinaturas           → adhoc-signatures.json      (20% redução)
Ad-Hoc com lógica inline         → adhoc-logic.json           (10% redução)
```

## ⚡ Comandos Rápidos

### Usar uma config

```bash
# 1. Copiar config
cp examples/configs/llm-optimized.json ./ycg.config.json

# 2. Gerar índice SCIP
ycg index --directory . --output index.scip

# 3. Gerar grafo (usa config automaticamente)
ycg generate --input index.scip --output graph.yaml
```

### Testar todas as configs

```bash
# Testar
make test-configs

# Comparar resultados
make compare-configs

# Limpar
make clean-configs
```

### Comparar duas configs

```bash
# Gerar com config 1
cp examples/configs/minimal.json ./ycg.config.json
ycg generate --input index.scip --output graph-minimal.yaml

# Gerar com config 2
cp examples/configs/llm-optimized.json ./ycg.config.json
ycg generate --input index.scip --output graph-optimized.yaml

# Comparar
diff graph-minimal.yaml graph-optimized.yaml
wc -l graph-*.yaml
```

## 📊 Tabela de Comparação

| Config | Formato | Compact | Framework Noise | Granularity | Redução |
|--------|---------|---------|-----------------|-------------|---------|
| minimal | YAML | ❌ | ❌ | - | 0% |
| typescript-standard | YAML | ✅ | ✅ | - | ~50% |
| adhoc-default | Ad-Hoc | ❌ | ❌ | Default | ~30% |
| adhoc-signatures | Ad-Hoc | ❌ | ❌ | Signatures | ~20% |
| adhoc-logic | Ad-Hoc | ❌ | ❌ | Logic | ~10% |
| llm-optimized | Ad-Hoc | ✅ | ✅ | Logic | ~80% |
| documentation | Ad-Hoc | ❌ | ✅ | Signatures | ~40% |
| architecture-analysis | YAML | ✅ | ✅ | - | ~60% |

## 🔧 Customização Rápida

### Adicionar exclusões

```json
{
  "ignore": {
    "customPatterns": [
      "**/vendor/**",
      "**/third_party/**"
    ]
  }
}
```

### Incluir apenas src/

```json
{
  "include": [
    "src/**/*.ts",
    "src/**/*.tsx"
  ]
}
```

### Desativar compactação

```json
{
  "output": {
    "compact": false
  }
}
```

### Mudar formato

```json
{
  "output": {
    "format": "adhoc"  // ou "yaml"
  }
}
```

### Mudar granularidade

```json
{
  "output": {
    "format": "adhoc",
    "adhocGranularity": "logic"  // "default", "signatures", ou "logic"
  }
}
```

## 🎓 Exemplos por Cenário

### Cenário 1: Enviar para GPT-4

```bash
cp examples/configs/llm-optimized.json ./ycg.config.json
ycg generate --input index.scip --output context.yaml
# Resultado: ~80% menos tokens
```

### Cenário 2: Documentar API

```bash
cp examples/configs/documentation.json ./ycg.config.json
ycg generate --input index.scip --output api-docs.yaml
# Resultado: Assinaturas legíveis, sem boilerplate
```

### Cenário 3: Analisar dependências

```bash
cp examples/configs/architecture-analysis.json ./ycg.config.json
ycg generate --input index.scip --lod 0 --output architecture.yaml
# Resultado: Apenas estrutura de alto nível
```

### Cenário 4: Comparar formatos

```bash
# YAML
cp examples/configs/minimal.json ./ycg.config.json
ycg generate --input index.scip --output graph.yaml

# Ad-Hoc
cp examples/configs/adhoc-default.json ./ycg.config.json
ycg generate --input index.scip --output graph-adhoc.yaml

# Comparar
wc -l graph*.yaml
```

## 🐛 Troubleshooting Rápido

### Config não carrega

```bash
# Verificar se existe
ls -la ycg.config.json

# Validar JSON
cat ycg.config.json | jq .
```

### Output muito grande

```bash
# Use a config mais agressiva
cp examples/configs/llm-optimized.json ./ycg.config.json
```

### Faltando símbolos

```bash
# Desative compact
{
  "output": {
    "compact": false
  }
}

# Ou use LOD mais alto
ycg generate --input index.scip --lod 2
```

### Erro de formato

```bash
# Verifique se granularity requer adhoc
{
  "output": {
    "format": "adhoc",  // Obrigatório para granularity
    "adhocGranularity": "logic"
  }
}
```

## 📚 Links Úteis

- [README Completo](README.md) - Documentação detalhada
- [Outputs de Teste](outputs/README.md) - Exemplos gerados
- [Documentação Principal](../../README.md) - Docs do YCG

## 💡 Dicas

1. **Sempre teste primeiro**: Use `make test-configs` antes de usar em produção
2. **Compare outputs**: Use `make compare-configs` para ver diferenças
3. **CLI sobrescreve config**: Flags CLI têm precedência sobre arquivo
4. **Gitignore é respeitado**: Por padrão, arquivos em .gitignore são excluídos
5. **Compact é agressivo**: Remove ~50% dos símbolos, use com cuidado

---

**Última atualização**: Dezembro 2024
