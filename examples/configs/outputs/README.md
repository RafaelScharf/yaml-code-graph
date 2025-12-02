# YCG Configuration Test Outputs

Este diretório contém os outputs gerados pelos testes de todas as configurações do YCG.

## 📁 Estrutura

```
outputs/
├── simple-ts/              # Outputs do projeto simple-ts
│   ├── minimal.yaml
│   ├── typescript-standard.yaml
│   ├── adhoc-default.yaml
│   ├── adhoc-signatures.yaml
│   ├── adhoc-logic.yaml
│   ├── llm-optimized.yaml
│   ├── documentation.yaml
│   └── architecture-analysis.yaml
│
├── nestjs-api-ts/          # Outputs do projeto nestjs-api-ts
│   ├── minimal.yaml
│   ├── typescript-standard.yaml
│   ├── adhoc-default.yaml
│   ├── adhoc-signatures.yaml
│   ├── adhoc-logic.yaml
│   ├── llm-optimized.yaml
│   ├── documentation.yaml
│   └── architecture-analysis.yaml
│
├── metrics.csv             # Métricas de todos os testes
└── README.md               # Este arquivo
```

## 📊 Métricas

Veja `metrics.csv` para comparação detalhada de:
- Número de linhas
- Tamanho do arquivo
- Tokens aproximados
- Tempo de geração

## 🔄 Regenerar Outputs

```bash
# Da raiz do projeto
make test-configs

# Ou diretamente
./examples/configs/test-configs.sh
```

## 📖 Comparar Outputs

```bash
# Comparar dois configs
diff outputs/simple-ts/minimal.yaml outputs/simple-ts/llm-optimized.yaml

# Ver tamanhos
du -h outputs/simple-ts/*.yaml

# Contar linhas
wc -l outputs/simple-ts/*.yaml
```

## 🎯 Análise Rápida

### Menor Output (mais tokens economizados)
```bash
ls -lhS outputs/simple-ts/*.yaml | tail -1
```

### Maior Output (mais detalhado)
```bash
ls -lhS outputs/simple-ts/*.yaml | head -2 | tail -1
```

### Comparação de Tokens
```bash
# Aproximação: 1 token ≈ 4 caracteres
for f in outputs/simple-ts/*.yaml; do
    echo "$(basename $f): ~$(($(wc -c < $f) / 4)) tokens"
done | sort -t: -k2 -n
```

---

**Gerado automaticamente por**: `test-configs.sh`
