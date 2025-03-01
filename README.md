# Replica Compiler

![replica workflow](https://github.com/shotastage/Replica/actions/workflows/rust.yml/badge.svg)


>> This project is now under construction and does not work now!
>> Fllowing readme is generated by AI. Use caution to read in detail.

A WebAssembly-targeted compiler for the Replica programming language, designed for distributed actor-based systems.

## Overview

Replica is a modern programming language that combines actor-based concurrency with distributed computing capabilities. The language features:

- Swift/Java-like object-oriented syntax
- Actor-based concurrency model with distributed actors by default
- Ownership system similar to Rust for memory safety
- Async/await for parallel processing
- WebAssembly as the target platform
- Built-in support for distributed state management

## Key Features

### Actor System

All objects in Replica are actors by default:

```swift
actor Counter {
    var value: Int = 0

    func increment() async {
        value += 1
    }

    func getValue() async -> Int {
        return value
    }
}
```

### Single Actor Optimization

Use `single actor` for performance optimization when distribution isn't needed:

```swift
single actor Logger {
    let name: String

    immediate init(name: String) {
        self.name = name
    }

    func log(message: String) {
        print("[\(name)] \(message)")
    }
}
```

### Ownership System

Replica implements an ownership system for safe memory management:

- Default ownership for actor instances
- Move semantics for transferring ownership
- Copy operations for single actor to distributed actor conversion
- Shared state for distributed actor communication

## Project Structure

- `src/`
  - `main.rs` - Compiler entry point and orchestration
  - `lexer.rs` - Lexical analysis implementation
  - `parser.rs` - Syntax parser and AST builder
  - `ast.rs` - Abstract Syntax Tree definitions
  - `semantic.rs` - Semantic analysis and type checking
  - `codegen.rs` - WASM code generation using LLVM
  - `ownership.rs` - Ownership system implementation

## Building the Project

### Prerequisites

- Rust toolchain (latest stable version)
- LLVM development libraries
- WebAssembly target support

### Build Instructions

1. Clone the repository
```bash
git clone <repository-url>
cd replica-compiler
```

2. Build the project
```bash
cargo build --release
```

3. Run tests
```bash
cargo test
```

## Usage

### Basic Compilation

```bash
./replica-compiler <input_file.replica> <output_file.wasm>
```

### Example

```swift
// hello.replica
actor Greeter {
    let greeting: String

    immediate init(greeting: String) {
        self.greeting = greeting
    }

    func greet(name: String) -> String {
        return "\(greeting), \(name)!"
    }
}
```

Compile to WebAssembly:
```bash
./replica-compiler hello.replica hello.wasm
```

## Language Features

### Async/Await Support

Replica provides three types of asynchronous execution:

1. Standard async tasks
2. Sequential tasks (ordered execution)
3. Priority-based tasks

Example of sequential task:
```swift
actor Logger {
    sequential func logMessage(_ message: String) async {
        print("[LOG]: \(message)")
    }
}
```

### Ownership Models

| Operation | Single Actor | Distributed Actor |
|-----------|--------------|------------------|
| move | ❌ | ✅ |
| copy | ✅ (to distributed) | ❌ |
| shared | ❌ | ✅ |

## Contributing

Contributions are welcome! Please check the following guidelines:

1. Fork the repository
2. Create a feature branch
3. Write tests for new features
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License

Copyright (c) 2024 Shota Shimazu <shota.shimazu@mgq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Development Status

This project is currently in development. The following components are implemented:

✅ Lexical Analysis
✅ Syntax Parsing
✅ AST Generation
✅ Semantic Analysis
✅ Basic WASM Code Generation
✅ Ownership System (partial)

Upcoming features:
- Complete ownership system implementation
- Advanced optimization passes
- Standard library development
- IDE integration

## Acknowledgments

- Shota Shimazu - Initial developer and project maintainer
- All contributors who will help improve this project
