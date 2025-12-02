#!/bin/bash
# Test script for YCG configurations
# Tests all config files against example projects

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$EXAMPLES_DIR")"
OUTPUT_DIR="$SCRIPT_DIR/outputs"
YCG_BIN="$PROJECT_ROOT/target/release/ycg_cli"

# Test projects
SIMPLE_TS="$EXAMPLES_DIR/simple-ts"
NESTJS_API="$EXAMPLES_DIR/nestjs-api-ts"

# Config files to test
CONFIGS=(
    "minimal.json"
    "typescript-standard.json"
    "adhoc-default.json"
    "adhoc-signatures.json"
    "adhoc-logic.json"
    "llm-optimized.json"
    "documentation.json"
    "architecture-analysis.json"
)

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Function to check if YCG binary exists
check_ycg_binary() {
    if [ ! -f "$YCG_BIN" ]; then
        print_error "YCG binary not found at $YCG_BIN"
        print_info "Please build the project first: cargo build --release"
        exit 1
    fi
    print_success "Found YCG binary at $YCG_BIN"
}

# Function to check if SCIP index exists
check_scip_index() {
    local project_dir=$1
    local scip_file="$project_dir/index.scip"
    
    if [ ! -f "$scip_file" ]; then
        print_warning "SCIP index not found at $scip_file"
        print_info "Generating SCIP index..."
        
        # Check if it's a TypeScript project
        if [ -f "$project_dir/tsconfig.json" ]; then
            cd "$project_dir"
            npx @sourcegraph/scip-typescript index --output index.scip 2>&1 | grep -v "npm WARN" || true
            cd "$SCRIPT_DIR"
            
            if [ -f "$scip_file" ]; then
                print_success "Generated SCIP index for $(basename "$project_dir")"
            else
                print_error "Failed to generate SCIP index for $(basename "$project_dir")"
                return 1
            fi
        else
            print_error "Unknown project type for $(basename "$project_dir")"
            return 1
        fi
    else
        print_success "Found SCIP index at $scip_file"
    fi
}

# Function to count tokens (approximate)
count_tokens() {
    local file=$1
    # Approximate: 1 token ≈ 4 characters
    local chars=$(wc -c < "$file")
    echo $((chars / 4))
}

# Function to test a config on a project
test_config() {
    local config_file=$1
    local project_dir=$2
    local project_name=$(basename "$project_dir")
    local config_name=$(basename "$config_file" .json)
    
    print_info "Testing $config_name on $project_name..."
    
    # Create output directory for this project
    local project_output_dir="$OUTPUT_DIR/$project_name"
    mkdir -p "$project_output_dir"
    
    # Copy config to project directory
    cp "$SCRIPT_DIR/$config_file" "$project_dir/ycg.config.json"
    
    # Determine output file extension based on config
    local output_ext="yaml"
    if grep -q '"format".*"adhoc"' "$SCRIPT_DIR/$config_file"; then
        output_ext="yaml"  # Ad-hoc still outputs as YAML
    fi
    
    # Output file
    local output_file="$project_output_dir/${config_name}.${output_ext}"
    
    # Run YCG generate
    local start_time=$(date +%s)
    if "$YCG_BIN" generate \
        --input "$project_dir/index.scip" \
        --output "$output_file" \
        --root "$project_dir" \
        > /dev/null 2>&1; then
        
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        
        # Get file stats
        local lines=$(wc -l < "$output_file")
        local size=$(du -h "$output_file" | cut -f1)
        local tokens=$(count_tokens "$output_file")
        
        print_success "Generated: $output_file"
        echo "           Lines: $lines | Size: $size | Tokens: ~$tokens | Time: ${duration}s"
        
        # Store metrics
        echo "$config_name,$project_name,$lines,$size,$tokens,$duration" >> "$OUTPUT_DIR/metrics.csv"
    else
        print_error "Failed to generate output for $config_name on $project_name"
        return 1
    fi
    
    # Clean up config file
    rm -f "$project_dir/ycg.config.json"
}

# Function to generate comparison report
generate_report() {
    print_header "📊 Test Results Summary"
    
    if [ ! -f "$OUTPUT_DIR/metrics.csv" ]; then
        print_warning "No metrics file found"
        return
    fi
    
    echo ""
    printf "%-25s %-15s %10s %10s %12s %8s\n" "Config" "Project" "Lines" "Size" "Tokens" "Time"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    while IFS=',' read -r config project lines size tokens time; do
        printf "%-25s %-15s %10s %10s %12s %8ss\n" "$config" "$project" "$lines" "$size" "$tokens" "$time"
    done < "$OUTPUT_DIR/metrics.csv"
    
    echo ""
    
    # Calculate totals
    local total_files=$(wc -l < "$OUTPUT_DIR/metrics.csv")
    print_success "Generated $total_files output files"
    
    # Find most efficient config
    local min_tokens=$(awk -F',' '{print $5}' "$OUTPUT_DIR/metrics.csv" | sort -n | head -1)
    local most_efficient=$(grep ",$min_tokens," "$OUTPUT_DIR/metrics.csv" | cut -d',' -f1 | head -1)
    print_info "Most efficient config: $most_efficient (~$min_tokens tokens)"
}

# Function to create README in outputs directory
create_output_readme() {
    cat > "$OUTPUT_DIR/README.md" << 'EOF'
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
EOF
    
    print_success "Created README in $OUTPUT_DIR"
}

# Main execution
main() {
    print_header "🧪 YCG Configuration Testing Suite"
    
    print_info "Script directory: $SCRIPT_DIR"
    print_info "Project root: $PROJECT_ROOT"
    print_info "Output directory: $OUTPUT_DIR"
    
    # Check prerequisites
    check_ycg_binary
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    
    # Initialize metrics file
    echo "config,project,lines,size,tokens,time" > "$OUTPUT_DIR/metrics.csv"
    
    # Test simple-ts project
    print_header "📦 Testing simple-ts project"
    check_scip_index "$SIMPLE_TS"
    
    for config in "${CONFIGS[@]}"; do
        test_config "$config" "$SIMPLE_TS" || print_warning "Skipping $config on simple-ts"
    done
    
    # Test nestjs-api-ts project
    print_header "📦 Testing nestjs-api-ts project"
    check_scip_index "$NESTJS_API"
    
    for config in "${CONFIGS[@]}"; do
        test_config "$config" "$NESTJS_API" || print_warning "Skipping $config on nestjs-api-ts"
    done
    
    # Generate report
    generate_report
    
    # Create README
    create_output_readme
    
    print_header "🎉 All tests completed!"
    print_info "Outputs saved to: $OUTPUT_DIR"
    print_info "View metrics: cat $OUTPUT_DIR/metrics.csv"
}

# Run main function
main "$@"
