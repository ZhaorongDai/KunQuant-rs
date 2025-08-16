use crate::error::{KunQuantError, Result};
use crate::ffi;
use std::collections::HashMap;
use std::ffi::CString;

/// A mapping from buffer names to memory buffers for KunQuant computation.
///
/// `BufferNameMap` provides the interface between Rust memory and KunQuant's
/// computation engine. It manages named buffers that serve as inputs and outputs
/// for factor computations, ensuring memory safety and proper lifetime management.
///
/// # Buffer Management
///
/// - Input buffers contain market data (prices, volumes, etc.)
/// - Output buffers store computed factor values
/// - Buffers are referenced by name as defined in the factor module
/// - Memory layout must match KunQuant's expectations (row-major, f32 values)
///
/// # Memory Safety
///
/// The buffer map maintains references to ensure that:
/// - Buffer memory remains valid during computation
/// - C strings for buffer names are not deallocated prematurely
/// - No use-after-free errors occur when accessing buffers
///
/// # Thread Safety
///
/// This struct is not thread-safe. Each thread should create its own
/// `BufferNameMap` instance for concurrent computations.
pub struct BufferNameMap {
    handle: ffi::KunBufferNameMapHandle,
    // Keep track of buffer names to prevent use-after-free
    _buffer_names: HashMap<String, CString>,
}

impl BufferNameMap {
    /// Creates a new empty buffer name map.
    ///
    /// This initializes the internal data structures needed to manage
    /// named buffer mappings for KunQuant computations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(BufferNameMap)` on success, or
    /// `Err(KunQuantError::BufferNameMapCreationFailed)` if the underlying
    /// C library fails to create the buffer map.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::BufferNameMap;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// let mut buffers = BufferNameMap::new()?;
    ///
    /// // Now ready to add buffer mappings
    /// // buffers.set_buffer_slice("close", &mut price_data)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Memory Usage
    ///
    /// The initial buffer map has minimal memory overhead. Memory usage
    /// grows as buffers are added, but the map itself doesn't copy buffer data.
    pub fn new() -> Result<Self> {
        let handle = unsafe { ffi::kunCreateBufferNameMap() };
        if handle.is_null() {
            return Err(KunQuantError::BufferNameMapCreationFailed);
        }

        Ok(BufferNameMap {
            handle,
            _buffer_names: HashMap::new(),
        })
    }

    /// Sets a buffer mapping using a raw pointer (unsafe).
    ///
    /// This method directly maps a buffer name to a raw memory pointer.
    /// It's primarily used internally and by advanced users who need
    /// direct memory control.
    ///
    /// # Arguments
    ///
    /// * `name` - The buffer name as defined in the factor module
    /// * `buffer` - Raw pointer to the buffer memory
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - The buffer must remain valid for the lifetime of this `BufferNameMap`
    /// - The buffer must be large enough to hold the expected data
    /// - The pointer must be properly aligned for f32 values
    /// - The caller must ensure no data races occur during computation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::BufferNameMap;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// let mut buffers = BufferNameMap::new()?;
    /// let mut data = vec![1.0f32; 1000];
    ///
    /// unsafe {
    ///     buffers.set_buffer("close", data.as_mut_ptr())?;
    /// }
    ///
    /// // data must remain valid until buffers is dropped
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Preferred Alternative
    ///
    /// Consider using `set_buffer_slice()` instead, which provides the same
    /// functionality with compile-time safety guarantees.
    pub unsafe fn set_buffer<N: AsRef<str>>(&mut self, name: N, buffer: *mut f32) -> Result<()> {
        let name_str = name.as_ref();
        let c_name = CString::new(name_str)?;

        unsafe {
            ffi::kunSetBufferNameMap(self.handle, c_name.as_ptr(), buffer);
        }
        self._buffer_names.insert(name_str.to_string(), c_name);

        Ok(())
    }

    /// Sets a buffer mapping using a mutable slice (safe).
    ///
    /// This is the recommended way to map buffers as it provides compile-time
    /// safety guarantees. The slice must remain valid for the lifetime of
    /// the `BufferNameMap`.
    ///
    /// # Arguments
    ///
    /// * `name` - The buffer name as defined in the factor module. Can be any
    ///           type that implements `AsRef<str>` (e.g., `&str`, `String`, etc.)
    /// * `buffer` - Mutable slice containing the buffer data
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if:
    /// - The buffer name contains null bytes
    /// - The underlying C library call fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::BufferNameMap;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// let mut buffers = BufferNameMap::new()?;
    ///
    /// // Set up input data (16 stocks × 100 time points)
    /// let mut close_prices = vec![100.0f32; 1600];
    /// let mut volumes = vec![1000.0f32; 1600];
    ///
    /// // Map input buffers
    /// buffers.set_buffer_slice("close", &mut close_prices)?;
    /// buffers.set_buffer_slice("volume", &mut volumes)?;
    ///
    /// // Set up output buffer
    /// let mut factor_output = vec![0.0f32; 1600];
    /// buffers.set_buffer_slice("my_factor", &mut factor_output)?;
    ///
    /// // Buffers are now ready for computation
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Data Layout Requirements
    ///
    /// - Data must be in row-major order: `[t0_s0, t0_s1, ..., t0_sN, t1_s0, ...]`
    /// - Buffer size must match `num_stocks * total_time`
    /// - All values should be finite floating-point numbers
    /// - Input buffers should be populated before computation
    /// - Output buffers will be overwritten during computation
    ///
    /// # Memory Management
    ///
    /// - The slice must remain valid until the `BufferNameMap` is dropped
    /// - No copying occurs - the buffer map holds references to your data
    /// - Ensure the slice is not moved or reallocated during computation
    pub fn set_buffer_slice<N: AsRef<str>>(&mut self, name: N, buffer: &mut [f32]) -> Result<()> {
        unsafe { self.set_buffer(name, buffer.as_mut_ptr()) }
    }

    /// Remove a buffer mapping
    pub fn erase_buffer<N: AsRef<str>>(&mut self, name: N) -> Result<()> {
        let name_str = name.as_ref();
        if let Some(c_name) = self._buffer_names.get(name_str) {
            unsafe {
                ffi::kunEraseBufferNameMap(self.handle, c_name.as_ptr());
            }
            self._buffer_names.remove(name_str);
        }
        Ok(())
    }

    /// Get the raw handle (for internal use)
    pub(crate) fn handle(&self) -> ffi::KunBufferNameMapHandle {
        self.handle
    }
}

impl Drop for BufferNameMap {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::kunDestoryBufferNameMap(self.handle);
            }
        }
    }
}

impl Default for BufferNameMap {
    fn default() -> Self {
        Self::new().expect("Failed to create BufferNameMap")
    }
}
