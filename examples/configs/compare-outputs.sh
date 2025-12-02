#!/bin/bash
# Compare outputs from different configurations

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR/outputs"

print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Check if outputs exist
if [ ! -d "$OUTPUT_DIR" ]; then
    echo -e "${RED}❌ No outputs found. Run 'make test-configs' first.${NC}"
    exit 1
fi

# Function to show file stats
show_stats() {
    local file=$1
    local name=$(basename "$file" .yaml)
    local lines=$(wc -l < "$file" 2>/dev/null || echo "0")
    local size=$(du -h "$file" 2>/dev/null | cut -f1 || echo "0")
    local tokens=$(($(wc -c < "$file" 2>/dev/null || echo "0") / 4))
    
    printf "%-30s %10s lines | %8s | ~%10s tokens\n" "$name" "$lines" "$size" "$tokens"
}

# Main comparison
main() {
    print_header "📊 YCG Configuration Comparison"
    
    # Compare simple-ts outputs
    if [ -d "$OUTPUT_DIR/simple-ts" ]; then
        print_header "📦 simple-ts Project"
        echo ""
        printf "%-30s %10s        | %8s | %20s\n" "Configuration" "Lines" "Size" "Tokens (approx)"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        for file in "$OUTPUT_DIR/simple-ts"/*.yaml; do
            [ -f "$file" ] && show_stats "$file"
        done
    fi
    
    # Compare nestjs-api-ts outputs
    if [ -d "$OUTPUT_DIR/nestjs-api-ts" ]; then
        print_header "📦 nestjs-api-ts Project"
        echo ""
        printf "%-30s %10s        | %8s | %20s\n" "Configuration" "Lines" "Size" "Tokens (approx)"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        for file in "$OUTPUT_DIR/nestjs-api-ts"/*.yaml; do
            [ -f "$file" ] && show_stats "$file"
        done
    fi
    
    # Show reduction percentages
    print_header "📉 Token Reduction (vs minimal.json baseline)"
    echo ""
    
    for project in "simple-ts" "nestjs-api-ts"; do
        if [ -d "$OUTPUT_DIR/$project" ]; then
            echo -e "${BLUE}$project:${NC}"
            
            local baseline_file="$OUTPUT_DIR/$project/minimal.yaml"
            if [ -f "$baseline_file" ]; then
                local baseline_tokens=$(($(wc -c < "$baseline_file") / 4))
                
                for file in "$OUTPUT_DIR/$project"/*.yaml; do
                    [ -f "$file" ] || continue
                    local name=$(basename "$file" .yaml)
                    [ "$name" = "minimal" ] && continue
                    
                    local tokens=$(($(wc -c < "$file") / 4))
                    local reduction=$(( (baseline_tokens - tokens) * 100 / baseline_tokens ))
                    
                    if [ $reduction -gt 0 ]; then
                        echo -e "  ${GREEN}$name: -${reduction}% (${tokens} tokens)${NC}"
                    elif [ $reduction -lt 0 ]; then
                        local increase=$(( -reduction ))
                        echo -e "  ${YELLOW}$name: +${increase}% (${tokens} tokens)${NC}"
                    else
                        echo -e "  $name: 0% (${tokens} tokens)"
                    fi
                done
            fi
            echo ""
        fi
    done
    
    # Show recommendations
    print_header "🎯 Recommendations"
    echo ""
    print_info "For LLMs (maximum compression):"
    echo "  → llm-optimized.json"
    echo ""
    print_info "For documentation (readable):"
    echo "  → documentation.json"
    echo ""
    print_info "For architecture analysis:"
    echo "  → architecture-analysis.json"
    echo ""
    print_info "For TypeScript projects:"
    echo "  → typescript-standard.json"
    echo ""
    
    print_success "Comparison complete!"
    print_info "View detailed metrics: cat $OUTPUT_DIR/metrics.csv"
}

main "$@"
