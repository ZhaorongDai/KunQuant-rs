use thiserror::Error;

/// Comprehensive error types for KunQuant operations.
///
/// This enum covers all possible error conditions that can occur when using
/// the KunQuant-rs library, from initialization failures to runtime computation
/// errors. Each variant provides detailed context to help with debugging.
///
/// # Error Categories
///
/// - **Initialization Errors**: Executor, library, and buffer map creation failures
/// - **Resource Errors**: Library loading, module lookup, and buffer handle errors
/// - **Validation Errors**: Invalid parameters, buffer size mismatches, data constraints
/// - **Runtime Errors**: Null pointers, computation failures, and system errors
///
/// # Error Handling
///
/// All KunQuant operations return `Result<T, KunQuantError>`, allowing for
/// comprehensive error handling using Rust's `?` operator and pattern matching.
///
/// # Examples
///
/// ```rust,no_run
/// use kunquant_rs::{Executor, KunQuantError};
///
/// match Executor::single_thread() {
///     Ok(executor) => println!("Executor created successfully"),
///     Err(KunQuantError::ExecutorCreationFailed) => {
///         eprintln!("Failed to create executor - check KunQuant installation");
///     }
///     Err(e) => eprintln!("Unexpected error: {}", e),
/// }
/// ```
#[derive(Error, Debug)]
pub enum KunQuantError {
    /// Failed to create a KunQuant executor.
    ///
    /// This error occurs when the underlying C library cannot create an executor,
    /// typically due to insufficient system resources or library initialization issues.
    ///
    /// **Common Causes:**
    /// - KunQuant C library not properly installed
    /// - Insufficient memory for executor creation
    /// - Invalid thread configuration (for multi-threaded executors)
    /// - Missing required system dependencies
    #[error("Failed to create executor")]
    ExecutorCreationFailed,

    /// Failed to load a KunQuant factor library from the specified path.
    ///
    /// This error indicates that the library file could not be loaded, either
    /// because it doesn't exist, isn't accessible, or isn't a valid KunQuant library.
    ///
    /// **Common Causes:**
    /// - File doesn't exist at the specified path
    /// - Insufficient permissions to read the file
    /// - Library compiled for incompatible architecture
    /// - Missing shared library dependencies
    /// - Corrupted or invalid library file
    #[error("Failed to load library: {path}")]
    LibraryLoadFailed { path: String },

    /// The requested module was not found in the loaded library.
    ///
    /// This error occurs when trying to access a module that doesn't exist
    /// in the loaded library, typically due to incorrect module names or
    /// library compilation issues.
    ///
    /// **Common Causes:**
    /// - Typo in module name (names are case-sensitive)
    /// - Module not included during library compilation
    /// - Library compiled with different module names
    /// - Using wrong library file
    #[error("Module not found: {name}")]
    ModuleNotFound { name: String },

    /// Failed to create a buffer name map for data management.
    ///
    /// This error indicates that the internal buffer management system
    /// could not be initialized, typically due to memory allocation failures.
    ///
    /// **Common Causes:**
    /// - Insufficient system memory
    /// - Memory fragmentation
    /// - System resource limits exceeded
    #[error("Failed to create buffer name map")]
    BufferNameMapCreationFailed,

    /// The specified buffer name is invalid or contains illegal characters.
    ///
    /// Buffer names must be valid C strings without null bytes and should
    /// match the names defined in the factor module.
    ///
    /// **Common Causes:**
    /// - Buffer name contains null bytes ('\0')
    /// - Empty buffer name
    /// - Non-UTF8 characters in buffer name
    #[error("Invalid buffer name: {name}")]
    InvalidBufferName { name: String },

    /// The number of stocks is not a multiple of 8, which is required for SIMD optimization.
    ///
    /// KunQuant uses SIMD (Single Instruction, Multiple Data) instructions for
    /// performance optimization, which requires the stock count to be divisible by 8.
    ///
    /// **Solution:** Use a stock count that is a multiple of 8 (e.g., 8, 16, 24, 32, ...).
    #[error("Invalid number of stocks: {num_stocks}. Must be a multiple of 8")]
    InvalidStockCount { num_stocks: usize },

    /// Buffer size doesn't match the expected dimensions for the computation.
    ///
    /// This error occurs when the provided buffer size doesn't match the
    /// expected size based on the number of stocks and time points.
    ///
    /// **Expected Size:** `num_stocks * total_time` for time-series data
    #[error("Buffer size mismatch for '{name}': expected {expected}, got {actual}")]
    BufferSizeMismatch {
        name: String,
        expected: usize,
        actual: usize,
    },

    /// Failed to create a streaming computation context.
    ///
    /// This error occurs when the streaming context cannot be initialized,
    /// typically due to incompatible module settings or resource constraints.
    ///
    /// **Common Causes:**
    /// - Module not compiled with `output_layout="STREAM"`
    /// - Invalid number of stocks (not multiple of 8)
    /// - Insufficient memory for streaming buffers
    /// - Incompatible module and executor combination
    #[error("Stream context creation failed")]
    StreamCreationFailed,

    /// The requested buffer handle was not found in the streaming context.
    ///
    /// This error occurs when trying to access a buffer that hasn't been
    /// registered or doesn't exist in the current streaming context.
    ///
    /// **Common Causes:**
    /// - Buffer name doesn't match module definition
    /// - Buffer not properly initialized in streaming context
    /// - Typo in buffer name (names are case-sensitive)
    #[error("Buffer handle not found: {name}")]
    BufferHandleNotFound { name: String },

    /// A null pointer was encountered during C library interaction.
    ///
    /// This error indicates a serious internal issue where a C library
    /// function returned a null pointer unexpectedly.
    ///
    /// **Common Causes:**
    /// - Memory allocation failure in C library
    /// - Invalid handle passed to C function
    /// - Library corruption or version mismatch
    /// - System resource exhaustion
    #[error("Null pointer encountered")]
    NullPointer,

    /// Error converting Rust string to C string (contains null bytes).
    ///
    /// This error occurs when a Rust string contains null bytes ('\0'),
    /// which are not allowed in C strings used by the KunQuant library.
    ///
    /// **Solution:** Ensure strings don't contain null bytes.
    #[error("String conversion error: {0}")]
    StringConversion(#[from] std::ffi::NulError),

    /// Error converting C string to UTF-8.
    ///
    /// This error occurs when the C library returns a string that
    /// contains invalid UTF-8 sequences.
    ///
    /// **Common Causes:**
    /// - Corrupted data from C library
    /// - Encoding mismatch between C and Rust
    /// - Memory corruption
    #[error("UTF-8 conversion error: {0}")]
    Utf8Conversion(#[from] std::str::Utf8Error),
}

/// Type alias for Results using KunQuantError.
///
/// This is a convenience type alias that simplifies function signatures
/// throughout the KunQuant-rs library. Instead of writing
/// `std::result::Result<T, KunQuantError>`, you can simply use `Result<T>`.
///
/// # Examples
///
/// ```rust,no_run
/// use kunquant_rs::Result;
///
/// fn create_executor() -> Result<()> {
///     // Function implementation
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, KunQuantError>;
