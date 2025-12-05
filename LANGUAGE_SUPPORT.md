# YCG Language Support Status

This document provides detailed information about language support in YCG, including current status, known limitations, and future plans.

## Support Matrix

| Language | Status | SCIP Indexer | Tree-sitter | Logic Lifting | Recommended Use |
|----------|--------|--------------|-------------|---------------|-----------------|
| **TypeScript** | ✅ Stable | `@sourcegraph/scip-typescript` | ✅ Full | ✅ Full | Production |
| **JavaScript** | ✅ Stable | `@sourcegraph/scip-typescript` | ✅ Full | ✅ Full | Production |
| **Rust** | 🚧 Beta | `rust-analyzer` | ✅ Full | ⚠️ Partial | Testing/Development |
| **Python** | 📅 Planned | `scip-python` | 🔄 In Progress | 📅 Planned | Not Available |
| **Java** | 📅 Planned | `scip-java` | 🔄 In Progress | 📅 Planned | Not Available |

## Legend

- ✅ **Stable**: Production-ready, extensively tested, full feature support
- 🚧 **Beta**: Functional, core features work, may have edge cases
- 📅 **Planned**: On roadmap, not yet implemented
- 🔄 **In Progress**: Active development
- ⚠️ **Partial**: Works but with known limitations

## Detailed Status

### TypeScript (✅ Stable)

**Status**: Production-ready  
**Indexer**: `@sourcegraph/scip-typescript`  
**Tree-sitter Grammar**: Full support  
**Last Tested**: v0.2.0 (December 2024)

**Features**:
- ✅ Full symbol resolution
- ✅ Type inference and signatures
- ✅ Logic lifting (guard clauses, preconditions)
- ✅ Framework noise reduction (NestJS, TypeORM)
- ✅ Decorator handling
- ✅ Generic types
- ✅ Union/Intersection types
- ✅ Async/await patterns

**Known Limitations**: None

**Installation**:
```bash
npm install -g @sourcegraph/scip-typescript
```

**Example Projects**:
- `examples/nestjs-api-ts/` - Full NestJS application
- `examples/simple-ts/` - Minimal TypeScript example

**Recommended For**:
- Production codebases
- CI/CD integration
- LLM context generation
- API documentation
- Security audits

---

### JavaScript (✅ Stable)

**Status**: Production-ready  
**Indexer**: `@sourcegraph/scip-typescript` (same as TypeScript)  
**Tree-sitter Grammar**: Full support  
**Last Tested**: v0.2.0 (December 2024)

**Features**:
- ✅ Full symbol resolution
- ✅ JSDoc type annotations
- ✅ Logic lifting
- ✅ ES6+ syntax support
- ✅ Module systems (CommonJS, ESM)

**Known Limitations**: 
- Type information limited to JSDoc annotations
- Less precise than TypeScript for type inference

**Installation**:
```bash
npm install -g @sourcegraph/scip-typescript
```

**Recommended For**:
- Production JavaScript codebases
- Node.js applications
- Frontend frameworks (React, Vue, etc.)

---

### Rust (🚧 Beta)

**Status**: Beta - Functional but with edge cases  
**Indexer**: `rust-analyzer`  
**Tree-sitter Grammar**: Full support  
**Last Tested**: v0.2.0 (December 2024)

**Features**:
- ✅ Basic symbol resolution
- ✅ Function signatures
- ✅ Struct/Enum definitions
- ✅ Trait implementations
- ⚠️ Logic lifting (partial - simple patterns only)
- ⚠️ Macro expansion (limited)

**Known Limitations**:
1. **Macro-Heavy Codebases**: Complex procedural macros may not be fully resolved
2. **Logic Lifting**: Only simple guard clauses are detected (no match expressions yet)
3. **Async Rust**: Async/await patterns may have incomplete logic extraction
4. **Generic Constraints**: Complex trait bounds may not be fully represented

**Installation**:
```bash
rustup component add rust-analyzer
```

**Example Projects**:
- YCG itself (dogfooding) - `cargo run -- index`

**Recommended For**:
- Development and testing
- Simple Rust projects
- Architecture analysis (structure is accurate)

**Not Recommended For**:
- Production CI/CD (until stable)
- Macro-heavy codebases (e.g., heavy use of `derive` macros)
- Security audits requiring complete logic extraction

**Improvement Roadmap**:
- [ ] Enhanced macro expansion support
- [ ] Match expression logic lifting
- [ ] Async pattern detection
- [ ] Complex trait bound representation
- [ ] Comprehensive test suite

---

### Python (📅 Planned)

**Status**: Planned for v2.0  
**Target Release**: Q2 2025  
**Indexer**: `scip-python`  
**Tree-sitter Grammar**: In progress

**Planned Features**:
- Symbol resolution
- Type hints support
- Decorator handling
- Logic lifting for common patterns
- Framework support (Django, FastAPI)

**Installation**: Not yet available

**Tracking**: See [GitHub Issue #XX](https://github.com/yourusername/ycg/issues/XX)

---

### Java (📅 Planned)

**Status**: Planned for v2.0  
**Target Release**: Q3 2025  
**Indexer**: `scip-java`  
**Tree-sitter Grammar**: In progress

**Planned Features**:
- Symbol resolution
- Annotation handling
- Spring Framework support
- Logic lifting

**Installation**: Not yet available

**Tracking**: See [GitHub Issue #XX](https://github.com/yourusername/ycg/issues/XX)

---

## Choosing the Right Language

### For Production Use

**Recommended**: TypeScript or JavaScript

- Extensively tested
- Full feature support
- Active maintenance
- Large example projects

### For Experimentation

**Available**: Rust (Beta)

- Core functionality works
- Good for architecture analysis
- May have edge cases in complex codebases

### For Future Projects

**Coming Soon**: Python, Java

- Check roadmap for updates
- Consider contributing to development

---

## Testing Your Language Support

### Quick Test

```bash
# Navigate to your project
cd my-project

# Try automatic detection
ycg index

# If successful, generate graph
ycg generate -i index.scip -o graph.yaml --compact
```

### Validation Checklist

- [ ] SCIP index generated without errors
- [ ] Symbol definitions present in output
- [ ] Graph edges connect correctly
- [ ] Signatures extracted accurately
- [ ] Logic lifting works (if applicable)

### Reporting Issues

If you encounter language-specific issues:

1. Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
2. Verify indexer installation
3. Test with minimal example
4. Report on [GitHub Issues](https://github.com/yourusername/ycg/issues)

Include:
- Language and version
- YCG version
- Indexer version
- Minimal reproduction case

---

## Contributing Language Support

Interested in adding support for a new language?

### Requirements

1. **SCIP Indexer**: Must exist for the target language
2. **Tree-sitter Grammar**: Must be available
3. **Test Suite**: Comprehensive examples needed

### Process

1. Open a GitHub issue proposing the language
2. Implement Tree-sitter integration
3. Add logic lifting patterns
4. Create example projects
5. Write tests
6. Submit PR

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Version History

### v0.2.0 (December 2024)
- ✅ TypeScript: Stable
- ✅ JavaScript: Stable
- 🚧 Rust: Beta

### v0.1.0 (November 2024)
- 🚧 TypeScript: Beta
- 🚧 Rust: Alpha

---

## FAQ

### Q: Why is TypeScript recommended over Rust?

**A**: TypeScript support has been extensively tested in production environments, including large NestJS applications. Rust support is functional but hasn't undergone the same level of real-world validation.

### Q: When will Rust support be stable?

**A**: Rust support will move to stable once:
1. Macro expansion is fully supported
2. Logic lifting covers all common patterns
3. Comprehensive test suite passes
4. Production validation complete

Target: Q1 2025

### Q: Can I use YCG with Python today?

**A**: No, Python support is not yet implemented. It's planned for v2.0 (Q2 2025).

### Q: What about other languages (Go, C++, etc.)?

**A**: We prioritize languages based on:
1. SCIP indexer availability
2. Community demand
3. Tree-sitter grammar maturity

Open an issue to request support for your language.

---

## Support

- **Documentation**: [README.md](README.md)
- **Issues**: [GitHub Issues](https://github.com/yourusername/ycg/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/ycg/discussions)

---

**Last Updated**: December 2024  
**Version**: 0.2.0
