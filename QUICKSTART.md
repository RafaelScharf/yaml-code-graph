# YCG Quick Start Guide

Get started with YCG in 5 minutes!

## Installation

```bash
git clone https://github.com/yourusername/ycg.git
cd ycg
cargo build --release
sudo ./install.sh
```

## Your First YCG Graph

### 1. Create a Simple TypeScript Project

```bash
mkdir my-test-project
cd my-test-project
npm init -y
npm install -D @sourcegraph/scip-typescript
```

### 2. Create a Sample File

Create `index.ts`:

```typescript
function greet(name: string): string {
  if (name.length === 0) {
    throw new Error("Name cannot be empty");
  }
  return `Hello, ${name}!`;
}

class User {
  constructor(public name: string, public age: number) {}
  
  isAdult(): boolean {
    return this.age >= 18;
  }
}

const user = new User("Alice", 25);
console.log(greet(user.name));
```

### 3. Generate SCIP Index and YCG Graph

**Option A: Automatic (Recommended)**

```bash
# YCG automatically detects TypeScript and generates the index
ycg index
ycg generate -i index.scip -o graph.yaml --compact
```

**Option B: Manual**

```bash
# Generate SCIP index manually
npx scip-typescript index .

# Generate YCG graph
ycg generate -i index.scip -o graph.yaml --compact
```

### 5. View the Output

```bash
cat graph.yaml
```

You'll see a semantic graph like:

```yaml
_meta:
  name: ycg-v1.3
  version: 1.3.0

_defs:
  - id: greet_a3f2
    n: greet
    t: function
    sig: 'function greet(name: string): string'
    logic:
      pre:
        - 'must avoid: name.length === 0'
  
  - id: User_b8c1
    n: User
    t: class
    sig: 'class User'
  
  - id: isAdult_c4d5
    n: isAdult
    t: method
    parent_id: User_b8c1
    sig: 'isAdult(): boolean'

graph:
  greet_a3f2:
    calls:
      - Error_e9f3
  User_b8c1:
    references:
      - isAdult_c4d5
```

## Understanding the Output

### Metadata Section (`_meta`)
- Project name and version information

### Definitions Section (`_defs`)
- **id**: Unique deterministic identifier (semantic name + hash)
- **n**: Symbol name
- **t**: Symbol type (function, class, method, variable)
- **sig**: Full signature extracted by Tree-sitter
- **logic**: Extracted preconditions and invariants
- **parent_id**: Parent scope (for methods, nested functions)

### Graph Section
- **Compact mode**: Adjacency list format
- Maps each symbol to its outgoing edges
- Edge types: `calls`, `references`, `imports`

## Common Use Cases

### Use Case 1: Feed to LLM for Code Understanding

```bash
# One-command workflow
ycg index
ycg generate -i index.scip -o context.yaml --compact

# Use context.yaml as LLM context instead of raw code files
# Result: 60-75% fewer tokens with better semantic understanding
```

### Use Case 2: Analyze Large Codebase

```bash
# Generate index once
ycg index

# Low detail for overview (classes and functions only)
ycg generate -i index.scip -o overview.yaml --lod 0

# High detail for deep analysis
ycg generate -i index.scip -o detailed.yaml --lod 2
```

### Use Case 3: CI/CD Integration

```bash
#!/bin/bash
# generate-context.sh

# Automatic indexing and graph generation
ycg index -o index.scip
ycg generate -i index.scip -o docs/code-graph.yaml --compact

# Commit to repo for documentation
git add docs/code-graph.yaml
```

## Next Steps

- **Explore Examples**: Check out `examples/nestjs-api-ts` for a real-world project
- **Read the Docs**: See [README.md](README.md) for full documentation
- **Customize Output**: Experiment with different LOD levels
- **Integrate with LLMs**: Use the YAML output as context for GPT-4, Claude, etc.

## Tips & Tricks

### Tip 1: Compare Token Usage

```bash
# Count tokens in raw code
wc -w src/**/*.ts

# Count tokens in YCG output
wc -w graph.yaml

# YCG typically reduces by 60-75%
```

### Tip 2: Focus on Specific Files

```bash
# Generate SCIP for specific directory
ycg index -d ./src/core -o core-index.scip

# Process with YCG
ycg generate -i core-index.scip -o core-graph.yaml --compact
```

### Tip 3: Use with Git Hooks

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Auto-regenerate graph on commit
ycg index -o index.scip 2>/dev/null
if [ -f "index.scip" ]; then
  ycg generate -i index.scip -o .ycg-cache/graph.yaml --compact
fi
```

## Troubleshooting

### "Command not found: ycg"

Run the installation again:
```bash
sudo ./install.sh
```

### "Could not detect project language"

Make sure you have `Cargo.toml` (Rust) or `package.json` (TypeScript/JavaScript):
```bash
ls Cargo.toml package.json
```

### "rust-analyzer not found" or "scip-typescript not found"

Install the required indexer:
```bash
# For Rust
rustup component add rust-analyzer

# For TypeScript
npm install -g @sourcegraph/scip-typescript
```

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for detailed solutions.

### "Failed to read SCIP file"

Make sure you generated the index first:
```bash
ycg index
ls -la index.scip  # Should exist
```

### "Empty output"

Check your LOD level. Try `--lod 2` for maximum detail:
```bash
ycg generate -i index.scip -o graph.yaml --lod 2
```

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/ycg/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/ycg/discussions)
- **Documentation**: [Full README](README.md)
