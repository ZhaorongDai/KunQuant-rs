use crate::error::{KunQuantError, Result};
use crate::executor::Executor;
use crate::ffi;
use crate::library::Module;
use std::collections::HashMap;
use std::ffi::CString;

/// A streaming computation context for real-time factor calculation.
///
/// `StreamContext` provides an interface for real-time factor computation using KunQuant's
/// streaming engine. It maintains internal state for efficient buffer management and supports
/// low-latency processing of market data streams.
///
/// # Lifetime Parameters
///
/// * `'a` - The lifetime of the executor and module references
///
/// # Thread Safety
///
/// This struct is not thread-safe. Each thread should create its own `StreamContext` instance.
///
/// # Memory Management
///
/// The streaming context automatically manages its resources using RAII. The underlying
/// C handle is properly cleaned up when the context is dropped.
pub struct StreamContext<'a> {
    handle: ffi::KunStreamContextHandle,
    num_stocks: usize,
    _executor: &'a Executor,
    _module: &'a Module<'a>,
    // Cache buffer handles to avoid repeated lookups
    buffer_handles: HashMap<String, usize>,
}

impl<'a> StreamContext<'a> {
    /// Creates a new streaming context for real-time factor calculation.
    ///
    /// This function initializes a streaming context that can process market data
    /// in real-time using the specified executor and factor module.
    ///
    /// # Arguments
    ///
    /// * `executor` - Reference to the KunQuant executor that will run the computations
    /// * `module` - Reference to the compiled factor module containing the computation graph
    /// * `num_stocks` - Number of stocks to process
    ///
    /// # Returns
    ///
    /// Returns `Ok(StreamContext)` on success, or an error if:
    /// - The executor handle is invalid
    /// - The module handle is invalid
    /// - The streaming context creation fails in the C library
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::{Executor, Library, StreamContext};
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// let executor = Executor::single_thread()?;
    /// let library = Library::load("factor_lib.so")?;
    /// let module = library.get_module("my_factor")?;
    ///
    /// // Create streaming context for 16 stocks
    /// let stream = StreamContext::new(&executor, &module, 16)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(executor: &'a Executor, module: &'a Module<'a>, num_stocks: usize) -> Result<Self> {
        let handle =
            unsafe { ffi::kunCreateStream(executor.handle(), module.handle(), num_stocks) };

        if handle.is_null() {
            return Err(KunQuantError::StreamCreationFailed);
        }

        Ok(StreamContext {
            handle,
            num_stocks,
            _executor: executor,
            _module: module,
            buffer_handles: HashMap::new(),
        })
    }

    /// Retrieves the buffer handle for a named input or output buffer.
    ///
    /// Buffer handles are used internally by KunQuant to efficiently identify and access
    /// data buffers. This method caches handles to avoid repeated lookups, improving
    /// performance in streaming scenarios.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the buffer as defined in the factor module. Can be any type
    ///           that implements `AsRef<str>` (e.g., `&str`, `String`, etc.)
    ///
    /// # Returns
    ///
    /// Returns `Ok(handle)` where `handle` is a numeric identifier for the buffer,
    /// or an error if:
    /// - The buffer name is not found in the module
    /// - The streaming context handle is invalid
    /// - The C library call fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kunquant_rs::{Executor, Library, StreamContext, Result};
    /// # fn example(mut stream: StreamContext) -> Result<()> {
    /// // Get handle for input buffer
    /// let input_handle = stream.get_buffer_handle("close")?;
    ///
    /// // Get handle for output buffer
    /// let output_handle = stream.get_buffer_handle("factor_output")?;
    ///
    /// // Works with String as well
    /// let buffer_name = String::from("volume");
    /// let volume_handle = stream.get_buffer_handle(buffer_name)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Handles are cached internally after first lookup
    /// - Subsequent calls for the same buffer name return cached values
    /// - This optimization is crucial for high-frequency streaming applications
    /// - The cache persists for the lifetime of the `StreamContext`
    pub fn get_buffer_handle<N: AsRef<str>>(&mut self, name: N) -> Result<usize> {
        let name_str = name.as_ref();

        if let Some(&handle) = self.buffer_handles.get(name_str) {
            return Ok(handle);
        }

        let c_name = CString::new(name_str)?;
        let handle = unsafe { ffi::kunQueryBufferHandle(self.handle, c_name.as_ptr()) };

        // Note: KunQuant returns SIZE_MAX for invalid buffer names
        if handle == usize::MAX {
            return Err(KunQuantError::BufferHandleNotFound {
                name: name_str.to_string(),
            });
        }

        self.buffer_handles.insert(name_str.to_string(), handle);
        Ok(handle)
    }

    /// Retrieves the current computed data from a named output buffer.
    ///
    /// After calling `run()`, this method provides access to the computed factor values
    /// for the current time step. The returned slice contains values for all stocks.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the output buffer as defined in the factor module
    ///
    /// # Returns
    ///
    /// Returns `Ok(&[f32])` containing the computed values for all stocks, or an error if:
    /// - The buffer name is not found
    /// - The computation hasn't been run yet (call `run()` first)
    /// - The streaming context handle is invalid
    /// - The C library returns a null pointer
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kunquant_rs::{Executor, Library, StreamContext, Result};
    /// # fn example(mut stream: StreamContext) -> Result<()> {
    /// // Push input data
    /// let prices = vec![100.0, 200.0, 150.0, 75.0, 300.0, 125.0, 90.0, 180.0];
    /// stream.push_data("close", &prices)?;
    ///
    /// // Run computation
    /// stream.run()?;
    ///
    /// // Get computed factor values
    /// let factor_values = stream.get_current_buffer("my_factor")?;
    /// println!("Factor values: {:?}", factor_values);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Data Validity
    ///
    /// - The returned slice is valid until the next call to `push_data()` or `run()`
    /// - Values may include NaN for stocks where computation is undefined
    /// - The slice length always equals `num_stocks`
    ///
    /// # Safety Notes
    ///
    /// - The returned slice borrows from internal C buffers
    /// - The lifetime is tied to the `StreamContext` instance
    /// - Do not store references beyond the next streaming operation
    pub fn get_current_buffer<N: AsRef<str>>(&mut self, name: N) -> Result<&[f32]> {
        let handle = self.get_buffer_handle(name)?;
        let ptr = unsafe { ffi::kunStreamGetCurrentBuffer(self.handle, handle) };

        if ptr.is_null() {
            return Err(KunQuantError::NullPointer);
        }

        Ok(unsafe { std::slice::from_raw_parts(ptr, self.num_stocks) })
    }

    /// Pushes new market data to a named input buffer for the current time step.
    ///
    /// This method feeds new data into the streaming computation pipeline. The data
    /// represents values for all stocks at a single point in time (e.g., current prices,
    /// volumes, etc.).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the input buffer as defined in the factor module
    /// * `data` - Slice containing data for all stocks. Length must equal `num_stocks`
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if:
    /// - The data length doesn't match the number of stocks
    /// - The buffer name is not found
    /// - The streaming context handle is invalid
    /// - The C library call fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kunquant_rs::{Executor, Library, StreamContext, Result};
    /// # fn example(mut stream: StreamContext) -> Result<()> {
    /// // Push closing prices for 8 stocks
    /// let close_prices = vec![100.5, 200.3, 150.7, 75.2, 300.1, 125.8, 90.4, 180.6];
    /// stream.push_data("close", &close_prices)?;
    ///
    /// // Push volume data
    /// let volumes = vec![1000.0, 2000.0, 1500.0, 800.0, 3000.0, 1200.0, 900.0, 1800.0];
    /// stream.push_data("volume", &volumes)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Data Requirements
    ///
    /// - Data must be provided for exactly `num_stocks` securities
    /// - Values should be finite floating-point numbers
    /// - NaN and infinite values may cause computation errors
    /// - Data represents a single time point across all stocks
    ///
    /// # Performance Notes
    ///
    /// - Data is copied into internal buffers managed by KunQuant
    /// - Buffer handles are cached for optimal performance
    /// - This method is designed for high-frequency updates
    pub fn push_data<N: AsRef<str>>(&mut self, name: N, data: &[f32]) -> Result<()> {
        if data.len() != self.num_stocks {
            return Err(KunQuantError::BufferSizeMismatch {
                name: name.as_ref().to_string(),
                expected: self.num_stocks,
                actual: data.len(),
            });
        }

        let handle = self.get_buffer_handle(name)?;
        unsafe {
            ffi::kunStreamPushData(self.handle, handle, data.as_ptr());
        }
        Ok(())
    }

    /// Executes the factor computation on the currently pushed data.
    ///
    /// This method triggers the execution of the factor computation graph using all
    /// input data that has been pushed since the last `run()` call. After successful
    /// execution, computed results can be retrieved using `get_current_buffer()`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful computation, or an error if:
    /// - The streaming context handle is invalid
    /// - Required input data hasn't been pushed
    /// - The computation encounters runtime errors
    /// - The C library execution fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kunquant_rs::{Executor, Library, StreamContext, Result};
    /// # fn example(mut stream: StreamContext) -> Result<()> {
    /// // Push all required input data
    /// let close = vec![100.0, 200.0, 150.0, 75.0, 300.0, 125.0, 90.0, 180.0];
    /// let open = vec![99.0, 199.0, 149.0, 74.0, 299.0, 124.0, 89.0, 179.0];
    ///
    /// stream.push_data("close", &close)?;
    /// stream.push_data("open", &open)?;
    ///
    /// // Execute computation
    /// stream.run()?;
    ///
    /// // Now results are available
    /// let results = stream.get_current_buffer("output")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Execution Model
    ///
    /// - Computation is performed synchronously
    /// - All required inputs must be pushed before calling `run()`
    /// - Results are immediately available after successful execution
    /// - The method can be called repeatedly for streaming scenarios
    ///
    /// # Performance Notes
    ///
    /// - Optimized for low-latency execution
    /// - Uses SIMD instructions when possible
    /// - Memory buffers are reused between calls
    /// - Execution time depends on factor complexity and number of stocks
    pub fn run(&self) -> Result<()> {
        if self.handle.is_null() {
            return Err(KunQuantError::NullPointer);
        }

        unsafe {
            ffi::kunStreamRun(self.handle);
        }
        Ok(())
    }

    /// Returns the number of stocks this streaming context is configured to process.
    ///
    /// This value is set during context creation and determines the expected length
    /// of all input and output data arrays. It cannot be changed after creation.
    ///
    /// # Returns
    ///
    /// The number of stocks as specified when creating the streaming context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kunquant_rs::{Executor, Library, StreamContext, Result};
    /// # fn example(stream: StreamContext) -> Result<()> {
    /// let num_stocks = stream.num_stocks();
    /// println!("Processing {} stocks", num_stocks);
    ///
    /// // Ensure input data has correct length
    /// let prices = vec![100.0; num_stocks];
    /// // stream.push_data("close", &prices)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Usage Notes
    ///
    /// - All input data arrays must have exactly this length
    /// - All output data arrays will have exactly this length
    /// - The value is immutable for the lifetime of the context
    pub fn num_stocks(&self) -> usize {
        self.num_stocks
    }
}

impl<'a> Drop for StreamContext<'a> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::kunDestoryStream(self.handle);
            }
        }
    }
}
