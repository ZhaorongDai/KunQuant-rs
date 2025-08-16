use crate::error::{KunQuantError, Result};
use crate::ffi;

/// A KunQuant executor responsible for running factor computations.
///
/// The `Executor` manages the computational resources and threading model used
/// for factor calculations. It provides both single-threaded and multi-threaded
/// execution modes to optimize performance based on workload characteristics.
///
/// # Thread Safety
///
/// - The executor itself is thread-safe and can be shared across threads
/// - Multiple computations can be executed concurrently using the same executor
/// - Each computation maintains its own state and memory buffers
///
/// # Memory Management
///
/// The executor automatically manages its resources using RAII. The underlying
/// C handle is properly cleaned up when the executor is dropped.
///
/// # Performance Considerations
///
/// - Single-threaded executors have lower overhead for small computations
/// - Multi-threaded executors scale better for large factor libraries
/// - Thread count should typically match CPU core count for optimal performance
pub struct Executor {
    handle: ffi::KunExecutorHandle,
}

impl Executor {
    /// Creates a single-threaded executor optimized for low-latency computations.
    ///
    /// Single-threaded executors are ideal for:
    /// - Real-time streaming applications where latency is critical
    /// - Simple factors with low computational complexity
    /// - Scenarios where thread synchronization overhead outweighs benefits
    /// - Development and testing environments
    ///
    /// # Returns
    ///
    /// Returns `Ok(Executor)` on success, or `Err(KunQuantError::ExecutorCreationFailed)`
    /// if the underlying C library fails to create the executor.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::Executor;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// // Create a single-threaded executor for low-latency processing
    /// let executor = Executor::single_thread()?;
    ///
    /// // Use with streaming context for real-time factor calculation
    /// // let stream = StreamContext::new(&executor, &module, 16)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Minimal thread synchronization overhead
    /// - Predictable execution timing
    /// - Lower memory footprint compared to multi-threaded executors
    /// - Best suited for factors processing fewer than 1000 stocks
    pub fn single_thread() -> Result<Self> {
        let handle = unsafe { ffi::kunCreateSingleThreadExecutor() };
        if handle.is_null() {
            return Err(KunQuantError::ExecutorCreationFailed);
        }
        Ok(Executor { handle })
    }

    /// Creates a multi-threaded executor for high-throughput batch computations.
    ///
    /// Multi-threaded executors are ideal for:
    /// - Large-scale batch processing of historical data
    /// - Complex factors with high computational requirements
    /// - Processing thousands of stocks simultaneously
    /// - Scenarios where throughput is more important than latency
    ///
    /// # Arguments
    ///
    /// * `num_threads` - Number of worker threads to create. Should typically match
    ///                   the number of CPU cores for optimal performance. Values
    ///                   less than 1 will result in creation failure.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Executor)` on success, or `Err(KunQuantError::ExecutorCreationFailed)`
    /// if:
    /// - `num_threads` is less than 1
    /// - The underlying C library fails to create the executor
    /// - System resources are insufficient for the requested thread count
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::Executor;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// // Create executor with 4 worker threads
    /// let executor = Executor::multi_thread(4)?;
    ///
    /// // Optimal for CPU-bound batch processing
    /// let cpu_cores = std::thread::available_parallelism().unwrap().get() as i32;
    /// let optimal_executor = Executor::multi_thread(cpu_cores)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Scales well with CPU core count for compute-intensive factors
    /// - Higher memory usage due to per-thread buffers
    /// - Thread synchronization adds latency overhead
    /// - Best suited for batch processing of large datasets
    /// - Diminishing returns beyond CPU core count due to memory bandwidth limits
    pub fn multi_thread(num_threads: i32) -> Result<Self> {
        let handle = unsafe { ffi::kunCreateMultiThreadExecutor(num_threads) };
        if handle.is_null() {
            return Err(KunQuantError::ExecutorCreationFailed);
        }
        Ok(Executor { handle })
    }

    /// Get the raw handle (for internal use)
    pub(crate) fn handle(&self) -> ffi::KunExecutorHandle {
        self.handle
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::kunDestoryExecutor(self.handle);
            }
        }
    }
}

// Executor is thread-safe according to KunQuant documentation
unsafe impl Send for Executor {}
unsafe impl Sync for Executor {}
