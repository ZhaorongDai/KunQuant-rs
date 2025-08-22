# KunQuant-rs
[![Crates.io](https://img.shields.io/crates/v/kunquant_rs)](https://crates.io/crates/kunquant_rs)
[![Documentation](https://docs.rs/kunquant_rs/badge.svg)](https://docs.rs/kunquant_rs/0.2.1/kunquant_rs/)

Rust bindings for the KunQuant financial factor computation library.

## Overview

KunQuant-rs provides safe Rust bindings for [KunQuant](https://github.com/Menooker/KunQuant), a high-performance financial factor computation library. This crate wraps the C API with safe Rust abstractions while maintaining the performance characteristics of the underlying library.

## Features

- **Safe Rust API**: All unsafe FFI calls are wrapped in safe abstractions
- **RAII Resource Management**: Automatic cleanup of resources using Rust's Drop trait
- **Batch Computation**: Efficient batch processing of financial factors
- **Multi-threading Support**: Both single-thread and multi-thread executors
- **Streaming Computation**: Real-time factor calculation with low latency
- **Memory Safety**: Proper lifetime management and buffer handling

## Quick Start

### Dependencies

This project depends on the `KunRuntime` library. Follow these steps to ensure proper linking:

#### Linking `libKunRuntime.so`

1. **Compile with explicit library path**:
   ```bash
   cargo rustc -- -L /path/to/KunQuant/runner
   ```
   Replace `/path/to/KunQuant/runner` with the actual path to `libKunRuntime.so`.

2. **Permanent configuration via `RUSTFLAGS`**:
   ```bash
   export RUSTFLAGS="-L /path/to/KunQuant/runner"
   cargo build
   ```

3. **Runtime configuration**:
   Ensure the library is available at runtime by setting `LD_LIBRARY_PATH`:
   ```bash
   export LD_LIBRARY_PATH=/path/to/KunQuant/runner:$LD_LIBRARY_PATH
   ```

### Basic Usage

```rust
use kunquant_rs::{Executor, Library, BufferNameMap, BatchParams, run_graph};

// Create executor and load library
let executor = Executor::single_thread()?;
let library = Library::load("test_libs/simple_test_lib.so")?;
let module = library.get_module("simple_test")?;

// Prepare data
let mut input_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
let mut output_data = vec![0.0; 8];

// Set up buffers
let mut buffers = BufferNameMap::new()?;
buffers.set_buffer_slice("input", &mut input_data)?;
buffers.set_buffer_slice("output", &mut output_data)?;

// Run computation
let params = BatchParams::full_range(8, 1)?;
run_graph(&executor, &module, &buffers, &params)?;

// Results are now in output_data
println!("Results: {:?}", output_data);
```

## API Documentation

### Core Types

- `Executor`: Manages computation execution (single-thread or multi-thread)
- `Library`: Represents a loaded factor library
- `Module`: A specific factor module within a library
- `BufferNameMap`: Maps buffer names to data slices
- `BatchParams`: Parameters for batch computation
- `StreamContext`: Context for streaming computation

### Key Functions

- `run_graph()`: Execute a factor computation graph
- `Executor::single_thread()`: Create single-thread executor
- `Executor::multi_thread(n)`: Create multi-thread executor with n threads
- `Library::load(path)`: Load a factor library from file

## Testing

Run tests with the provided script that sets up the correct library path:

```bash
./run_tests.sh
```

## Architecture

The library follows a layered architecture:

1. **FFI Layer** (`ffi.rs`): Raw C bindings
2. **Error Handling** (`error.rs`): Rust error types
3. **Core Types** (`executor.rs`, `library.rs`): Safe wrappers
4. **Buffer Management** (`buffer.rs`): Memory-safe buffer handling
5. **Computation APIs** (`batch.rs`, `stream.rs`): High-level computation interfaces

## Memory Management

- All C resources are automatically cleaned up using Rust's RAII pattern
- Buffer lifetimes are tracked to prevent use-after-free
- No manual memory management required

## Performance

- Zero-cost abstractions over the C API
- Efficient buffer management with minimal copying
- Multi-threading support for parallel computation

## Limitations

- Requires KunQuant C library to be installed and accessible
- Factor libraries must be pre-compiled using the Python interface
- Streaming factors require special compilation with `output_layout="STREAM"`

## Contributing

1. Ensure all tests pass: `./run_tests.sh`
2. Add tests for new functionality
3. Follow Rust naming conventions and safety guidelines
4. Document public APIs

## License

MIT License - see LICENSE file for details.
