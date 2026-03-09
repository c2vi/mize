# Source code architecture

## 1. Core Mize Framework (`packages/mize/`)

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

## 3. MME - Mize Module Environment (`packages/mme/`)
- **Language**: Rust
- **Purpose**: Module execution environment
- **Key features**:
  - Cross-platform module loading
  - Qt and Slint UI support
  - Web view integration (tao/wry)
  - Command system (comandr)

## 4. PPC - Platform Components (`packages/ppc/`)
- **Language**: Rust + TypeScript
- **Purpose**: Platform-specific implementations
- **Targets**: Obsidian plugin, OS integration

## 5. Vic - Victor CLI (`packages/vic/`)
- **Language**: Rust
- **Purpose**: Command-line interface tool
- **Features**: Build, run, test, and manage Mize applications
