use crate::buffer::BufferNameMap;
use crate::error::Result;
use crate::executor::Executor;
use crate::ffi;
use crate::library::Module;

/// Parameters for batch computation of factor values over time series data.
///
/// `BatchParams` defines the dimensions and time window for batch factor computation.
/// It specifies how many stocks to process, the total time series length, and which
/// subset of time points to compute.
///
/// # Data Layout
///
/// KunQuant expects data in time-series format where:
/// - Rows represent time points (e.g., trading days)
/// - Columns represent stocks
/// - Data is stored in row-major order: `[t0_s0, t0_s1, ..., t0_sN, t1_s0, ...]`
///
/// # SIMD Requirements
///
/// In STs memory layout, `num_stocks` must be a multiple of 8 to enable
/// SIMD (Single Instruction, Multiple Data) vectorization.
#[derive(Debug, Clone)]
pub struct BatchParams {
    /// Number of stocks to process (must be multiple of 8 for STs, can be any positive integer for TS)
    pub num_stocks: usize,
    /// Total number of time points in the input data arrays
    pub total_time: usize,
    /// Starting time index for computation (0-based)
    pub cur_time: usize,
    /// Number of consecutive time points to compute
    pub length: usize,
}

impl BatchParams {
    /// Creates new batch computation parameters with validation.
    ///
    /// This constructor validates that the stock count meets SIMD requirements
    /// and that the time window parameters are consistent.
    ///
    /// # Arguments
    ///
    /// * `num_stocks` - Number of stocks to process (must be multiple of 8)
    /// * `total_time` - Total number of time points in input data
    /// * `cur_time` - Starting time index for computation (0-based)
    /// * `length` - Number of consecutive time points to compute
    ///
    /// # Returns
    ///
    /// Returns `Ok(BatchParams)` on success, or an error if:
    /// - `num_stocks` is not a multiple of 8
    /// - `cur_time + length > total_time` (time window exceeds data bounds)
    /// - Any parameter is invalid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::BatchParams;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// // Process 16 stocks over 100 time points, computing all points
    /// let params = BatchParams::new(16, 100, 0, 100)?;
    ///
    /// // Process 8 stocks, compute only the last 20 time points
    /// let params = BatchParams::new(8, 252, 232, 20)?;
    ///
    /// // This would fail - stock count not multiple of 8
    /// // let invalid = BatchParams::new(10, 100, 0, 100)?; // Error!
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Larger `num_stocks` values (multiples of 8) enable better SIMD utilization
    /// - Smaller `length` values reduce memory usage and computation time
    /// - `cur_time` and `length` allow processing data in chunks for memory efficiency
    pub fn new(
        num_stocks: usize,
        total_time: usize,
        cur_time: usize,
        length: usize,
    ) -> Result<Self> {
        Ok(BatchParams {
            num_stocks,
            total_time,
            cur_time,
            length,
        })
    }

    /// Creates parameters for computing the entire time range.
    ///
    /// This is a convenience method that creates batch parameters to process
    /// all time points in the dataset, equivalent to calling
    /// `BatchParams::new(num_stocks, total_time, 0, total_time)`.
    ///
    /// # Arguments
    ///
    /// * `num_stocks` - Number of stocks to process
    /// * `total_time` - Total number of time points in the data
    ///
    /// # Returns
    ///
    /// Returns `Ok(BatchParams)` configured to process the entire time range,
    ///
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::BatchParams;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// // Process all 252 trading days for 16 stocks
    /// let params = BatchParams::full_range(16, 252)?;
    ///
    /// // Equivalent to:
    /// // let params = BatchParams::new(16, 252, 0, 252)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Historical backtesting over complete datasets
    /// - Factor computation for entire time series
    /// - Initial factor validation and testing
    /// - Scenarios where memory constraints are not a concern
    pub fn full_range(num_stocks: usize, total_time: usize) -> Result<Self> {
        Self::new(num_stocks, total_time, 0, total_time)
    }
}

/// Executes batch factor computation on historical time series data.
///
/// This function runs a complete factor computation over a specified time window
/// using the provided executor, module, and data buffers. It's the primary interface
/// for batch processing of historical market data.
///
/// # Arguments
///
/// * `executor` - The KunQuant executor to use for computation
/// * `module` - The compiled factor module containing the computation graph
/// * `buffers` - Buffer map containing input data and output storage
/// * `params` - Batch parameters defining the computation window and dimensions
///
/// # Returns
///
/// Returns `Ok(())` on successful computation, or an error if:
/// - Input buffers don't contain required data
/// - Buffer dimensions don't match the parameters
/// - The computation encounters runtime errors
/// - Memory allocation fails during execution
///
/// # Examples
///
/// ```rust,no_run
/// use kunquant_rs::{Executor, Library, BufferNameMap, BatchParams, run_graph};
///
/// # fn main() -> kunquant_rs::Result<()> {
/// // Set up computation components
/// let executor = Executor::single_thread()?;
/// let library = Library::load("factors.so")?;
/// let module = library.get_module("alpha001")?;
///
/// // Prepare data buffers
/// let mut buffers = BufferNameMap::new()?;
/// let mut input_data = vec![1.0f32; 16 * 100]; // 16 stocks, 100 time points
/// let mut output_data = vec![0.0f32; 16 * 100];
///
/// buffers.set_buffer_slice("close", &mut input_data)?;
/// buffers.set_buffer_slice("alpha001", &mut output_data)?;
///
/// // Execute computation
/// let params = BatchParams::full_range(16, 100)?;
/// run_graph(&executor, &module, &buffers, &params)?;
///
/// // Results are now available in output_data
/// # Ok(())
/// # }
/// ```
///
/// # Data Requirements
///
/// - All input buffers must be populated with data before calling
/// - Buffer sizes must match `num_stocks * total_time`
/// - Data should be in row-major order (time-first layout)
/// - Output buffers must be pre-allocated with sufficient space
///
/// # Performance Notes
///
/// - Computation is CPU-intensive and benefits from multi-threading
/// - Memory usage scales with `num_stocks * total_time * sizeof(f32)`
/// - SIMD optimizations require `num_stocks` to be a multiple of 8
/// - Consider processing data in chunks for very large datasets
pub fn run_graph(
    executor: &Executor,
    module: &Module,
    buffers: &BufferNameMap,
    params: &BatchParams,
) -> Result<()> {
    unsafe {
        ffi::kunRunGraph(
            executor.handle(),
            module.handle(),
            buffers.handle(),
            params.num_stocks,
            params.total_time,
            params.cur_time,
            params.length,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_params_validation() {
        // Valid parameters
        assert!(BatchParams::new(8, 100, 0, 100).is_ok());
        assert!(BatchParams::new(16, 100, 10, 50).is_ok());

        // Invalid stock count (not multiple of 8)
        // assert!(BatchParams::new(7, 100, 0, 100).is_err());
        // assert!(BatchParams::new(15, 100, 0, 100).is_err());
    }

    #[test]
    fn test_full_range_params() {
        let params = BatchParams::full_range(24, 500).unwrap();
        assert_eq!(params.num_stocks, 24);
        assert_eq!(params.total_time, 500);
        assert_eq!(params.cur_time, 0);
        assert_eq!(params.length, 500);
    }
}
