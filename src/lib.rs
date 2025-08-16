//! # KunQuant-rs
//!
//! Rust bindings for the KunQuant financial factor computation library.
//!
//! KunQuant is an optimizer, code generator and executor for financial expressions
//! and factors. This crate provides safe Rust bindings to the KunQuant C API.
//!
//! ## Features
//!
//! - Batch mode computation for historical data analysis
//! - Stream mode computation for real-time factor calculation
//! - Thread-safe executors with single-thread and multi-thread support
//! - Memory-safe buffer management
//! - Support for both single and double precision floating point data
//!
//! ## Example
//!
//! ```rust,no_run
//! use kunquant_rs::{Executor, Library, BufferNameMap, BatchParams, run_graph, Result};
//!
//! fn main() -> Result<()> {
//!     // Create executor and load library
//!     let executor = Executor::single_thread()?;
//!     let library = Library::load("path/to/factor_library.so")?;
//!     let module = library.get_module("my_module")?;
//!
//!     // Set up input/output buffers
//!     let mut buffers = BufferNameMap::new()?;
//!     let mut input_data = vec![1.0f32; 8 * 100]; // 8 stocks, 100 time points
//!     let mut output_data = vec![0.0f32; 8 * 100];
//!
//!     buffers.set_buffer_slice("input", &mut input_data)?;
//!     buffers.set_buffer_slice("output", &mut output_data)?;
//!
//!     // Run computation
//!     let params = BatchParams::full_range(8, 100)?;
//!     run_graph(&executor, &module, &buffers, &params)?;
//!
//!     Ok(())
//! }
//! ```

pub mod batch;
pub mod buffer;
pub mod error;
pub mod executor;
pub mod ffi;
pub mod library;
pub mod stream;

// Re-export main types for convenience
pub use batch::{BatchParams, run_graph};
pub use buffer::BufferNameMap;
pub use error::{KunQuantError, Result};
pub use executor::Executor;
pub use library::{Library, Module};
pub use stream::StreamContext;
