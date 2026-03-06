# Mize Codebase Architecture Guide for AI Agents

## Overview
Mize is a strongly typed "filesystem" for the age of connectivity, elevating the Unix file philosophy into modern distributed computing.

## Codebase Structure

```
packages/
├── mize/          # Core Mize framework (Rust)
├── marts/         # Mize parts collection (Rust/TypeScript)
├── mme/           # Mize Module Environment (Rust)
├── ppc/           # Platform-specific components
├── vic/           # Victor CLI tool
└── ac_mize_macros/# Mize macros
```

## Key Components

### 1. Core Mize Framework (`packages/mize/`)
- **Language**: Rust
- **Target platforms**: OS (native), WASM (web), bare metal, JVM
- **Key features**:
  - Strongly typed filesystem abstraction
  - Module system with dynamic loading
  - Memory store implementation
  - Protocol versioning and serialization
  - Cross-platform support

**Architecture layers:**
- `core/`: Platform-independent logic
  - `config/`: Configuration management
  - `error/`: Error handling
  - `id/`: Unique identifier system
  - `instance/`: Mize runtime instances
  - `item/`: Filesystem item abstractions
  - `memstore/`: In-memory storage
  - `proto/`: Protocol definitions
  - `types/`: Core type definitions

- `platform/`: Platform-specific implementations
  - `os/`: Native OS integration
  - `wasm/`: WebAssembly support
  - `bare/`: Bare metal targets
  - `jvm/`: Java Virtual Machine integration

### 2. Marts (`packages/marts/`)
- **Language**: Rust + TypeScript
- **Purpose**: Collection of Mize framework parts and utilities
- **Key features**:
  - CLI tools
  - JavaScript/TypeScript integration
  - Habitica integration
  - Deno-based scripting support

### 3. MME - Mize Module Environment (`packages/mme/`)
- **Language**: Rust
- **Purpose**: Module execution environment
- **Key features**:
  - Cross-platform module loading
  - Qt and Slint UI support
  - Web view integration (tao/wry)
  - Command system (comandr)

### 4. PPC - Platform Components (`packages/ppc/`)
- **Language**: Rust + TypeScript
- **Purpose**: Platform-specific implementations
- **Targets**: Obsidian plugin, OS integration

### 5. Vic - Victor CLI (`packages/vic/`)
- **Language**: Rust
- **Purpose**: Command-line interface tool
- **Features**: Build, run, test, and manage Mize applications

## Build System

- **Rust**: Cargo workspace with feature flags for different targets
- **TypeScript**: Deno-based build system
- **Cross-compilation**: Support for WASM and multiple OS targets

## Key Technologies

- **Rust crates**: tokio, serde, wasm-bindgen, tracing, clap
- **WASM**: wasm-pack, web-sys
- **UI**: tao, wry, slint, Qt
- **Scripting**: Deno, rustyscript
- **Serialization**: ciborium (CBOR), serde_json, toml

## Development Patterns

1. **Feature flags**: Extensive use of Cargo features for platform targeting
2. **Cross-platform abstraction**: Core logic separated from platform implementations
3. **Module system**: Dynamic loading of functionality
4. **Strong typing**: Rust's type system used for filesystem safety

## Next Steps Discussion

The codebase appears to be in active development with these key areas for future work:

1. **Platform completion**: Finish OS, WASM, and other platform implementations
2. **Module ecosystem**: Develop more built-in modules
3. **Performance optimization**: Memory management and serialization
4. **Testing**: Comprehensive test coverage across platforms
5. **Documentation**: User-facing documentation and examples
6. **Integration**: Better integration with existing tools and systems

## Working with the Codebase

**For AI Agents:**
- Focus on the `core/` directory for platform-independent logic
- Use feature flags appropriately when building/testing
- Understand the module system for extending functionality
- Pay attention to cross-platform considerations

**Build targets:**
- `target-os`: Native operating system builds
- `target-wasm`: WebAssembly builds
- Other platform-specific targets as needed

## Key Files for Understanding

- `packages/mize/Cargo.toml`: Core dependencies and features
- `packages/mize/src/lib.rs`: Main library entry point
- `packages/mize/src/core/`: Core platform-independent logic
- `packages/mize/src/platform/`: Platform-specific implementations