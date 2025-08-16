use crate::error::{KunQuantError, Result};
use crate::ffi;
use std::ffi::CString;
use std::path::Path;

/// A loaded KunQuant library containing compiled factor modules.
///
/// A `Library` represents a dynamically loaded shared library (.so/.dll/.dylib)
/// that contains one or more compiled factor computation modules. These libraries
/// are typically generated using KunQuant's Python compilation interface.
///
/// # Library Structure
///
/// Each library can contain multiple named modules, where each module represents
/// a complete factor computation graph with defined inputs and outputs.
///
/// # Memory Management
///
/// The library automatically manages its resources using RAII. The underlying
/// C handle and loaded library are properly cleaned up when dropped.
///
/// # Thread Safety
///
/// Libraries are thread-safe and can be shared across multiple threads.
/// Multiple modules can be retrieved and used concurrently from the same library.
pub struct Library {
    handle: ffi::KunLibraryHandle,
}

impl Library {
    /// Loads a KunQuant factor library from the specified file path.
    ///
    /// This method dynamically loads a compiled factor library and makes its
    /// modules available for computation. The library must be compiled with
    /// compatible settings for the intended computation mode (batch vs streaming).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the compiled library file. Can be any type that implements
    ///           `AsRef<str>` (e.g., `&str`, `String`, `PathBuf`, etc.)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Library)` on successful loading, or an error if:
    /// - The file doesn't exist or isn't accessible
    /// - The file isn't a valid KunQuant library
    /// - The library has incompatible version or architecture
    /// - System resources are insufficient for loading
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::Library;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// // Load library from relative path
    /// let library = Library::load("factors/my_factors.so")?;
    ///
    /// // Load library from absolute path
    /// let library = Library::load("/opt/factors/alpha_factors.so")?;
    ///
    /// // Works with String as well
    /// let path = String::from("./test_factors.so");
    /// let library = Library::load(path)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Library Requirements
    ///
    /// - Must be compiled with compatible KunQuant version
    /// - Architecture must match the target platform
    /// - All dependencies must be available in the system
    /// - File must have appropriate read permissions
    ///
    /// # Performance Notes
    ///
    /// - Library loading is a one-time cost during initialization
    /// - Loaded libraries are cached by the system loader
    /// - Multiple `Library` instances of the same file share underlying resources
    pub fn load<P: AsRef<str>>(path: P) -> Result<Self> {
        if !Path::new(path.as_ref()).exists() {
            return Err(KunQuantError::LibraryLoadFailed {
                path: path.as_ref().to_string(),
            });
        }
        let path_str = path.as_ref();
        let c_path = CString::new(path_str)?;

        let handle = unsafe { ffi::kunLoadLibrary(c_path.as_ptr()) };
        if handle.is_null() {
            return Err(KunQuantError::LibraryLoadFailed {
                path: path_str.to_string(),
            });
        }

        Ok(Library { handle })
    }

    /// Retrieves a named factor module from the loaded library.
    ///
    /// Each library can contain multiple factor modules, each representing a
    /// complete computation graph with defined inputs and outputs. This method
    /// provides access to a specific module by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module as defined during compilation. Can be any
    ///           type that implements `AsRef<str>` (e.g., `&str`, `String`, etc.)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Module)` on success, or an error if:
    /// - No module with the specified name exists in the library
    /// - The library handle is invalid
    /// - The C library call fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kunquant_rs::Library;
    ///
    /// # fn main() -> kunquant_rs::Result<()> {
    /// let library = Library::load("factors.so")?;
    ///
    /// // Get a specific factor module
    /// let alpha001 = library.get_module("alpha001")?;
    /// let momentum = library.get_module("momentum_factor")?;
    ///
    /// // Works with String as well
    /// let module_name = String::from("mean_reversion");
    /// let mean_rev = library.get_module(module_name)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Module Naming
    ///
    /// - Module names are defined during factor compilation
    /// - Names are case-sensitive and must match exactly
    /// - Common naming conventions include factor names (e.g., "alpha001")
    ///   or descriptive names (e.g., "price_momentum", "volume_weighted_return")
    ///
    /// # Lifetime Management
    ///
    /// The returned `Module` maintains a reference to the parent `Library`,
    /// ensuring the library remains loaded for the module's lifetime.
    pub fn get_module<N: AsRef<str>>(&self, name: N) -> Result<Module> {
        let name_str = name.as_ref();
        let c_name = CString::new(name_str)?;

        let module_handle = unsafe { ffi::kunGetModuleFromLibrary(self.handle, c_name.as_ptr()) };
        if module_handle.is_null() {
            return Err(KunQuantError::ModuleNotFound {
                name: name_str.to_string(),
            });
        }

        Ok(Module {
            handle: module_handle,
            _library: self, // Keep library alive
        })
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::kunUnloadLibrary(self.handle);
            }
        }
    }
}

/// A KunQuant module containing factor computation logic.
///
/// A `Module` represents a compiled factor computation graph with defined inputs
/// and outputs. It encapsulates the mathematical operations, data flow, and
/// optimization strategies for a specific factor or set of related factors.
///
/// # Computation Modes
///
/// Modules can be compiled for different execution modes:
/// - **Batch Mode**: Optimized for processing historical time series data
/// - **Streaming Mode**: Optimized for real-time, low-latency computation
///
/// # Lifetime Parameters
///
/// * `'a` - The lifetime of the parent library reference
///
/// # Thread Safety
///
/// Modules are thread-safe and can be used concurrently across multiple threads.
/// Each computation context (batch or streaming) maintains its own state.
///
/// # Memory Management
///
/// The module maintains a reference to its parent library, ensuring the library
/// remains loaded for the module's entire lifetime. This prevents use-after-free
/// errors and ensures computation integrity.
pub struct Module<'a> {
    handle: ffi::KunModuleHandle,
    _library: &'a Library, // Keep library alive
}

impl<'a> Module<'a> {
    /// Get the raw handle (for internal use)
    pub(crate) fn handle(&self) -> ffi::KunModuleHandle {
        self.handle
    }
}
