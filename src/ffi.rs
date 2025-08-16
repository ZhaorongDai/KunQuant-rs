use libc::size_t;
use std::os::raw::{c_char, c_int, c_void};

// Opaque handle types from KunQuant C API
pub type KunExecutorHandle = *mut c_void;
pub type KunLibraryHandle = *mut c_void;
pub type KunModuleHandle = *mut c_void;
pub type KunBufferNameMapHandle = *mut c_void;
pub type KunStreamContextHandle = *mut c_void;

#[link(name = "KunRuntime")]
unsafe extern "C" {
    // Executor management
    pub fn kunCreateSingleThreadExecutor() -> KunExecutorHandle;
    pub fn kunCreateMultiThreadExecutor(numthreads: c_int) -> KunExecutorHandle;
    pub fn kunDestoryExecutor(ptr: KunExecutorHandle);

    // Library management
    pub fn kunLoadLibrary(path_or_name: *const c_char) -> KunLibraryHandle;
    pub fn kunGetModuleFromLibrary(lib: KunLibraryHandle, name: *const c_char) -> KunModuleHandle;
    pub fn kunUnloadLibrary(ptr: KunLibraryHandle);

    // Buffer name map management
    pub fn kunCreateBufferNameMap() -> KunBufferNameMapHandle;
    pub fn kunDestoryBufferNameMap(ptr: KunBufferNameMapHandle);
    pub fn kunSetBufferNameMap(ptr: KunBufferNameMapHandle, name: *const c_char, buffer: *mut f32);
    pub fn kunEraseBufferNameMap(ptr: KunBufferNameMapHandle, name: *const c_char);

    // Batch computation
    pub fn kunRunGraph(
        exec: KunExecutorHandle,
        m: KunModuleHandle,
        buffers: KunBufferNameMapHandle,
        num_stocks: size_t,
        total_time: size_t,
        cur_time: size_t,
        length: size_t,
    );

    // Stream computation
    pub fn kunCreateStream(
        exec: KunExecutorHandle,
        m: KunModuleHandle,
        num_stocks: size_t,
    ) -> KunStreamContextHandle;

    pub fn kunQueryBufferHandle(context: KunStreamContextHandle, name: *const c_char) -> size_t;

    pub fn kunStreamGetCurrentBuffer(context: KunStreamContextHandle, handle: size_t)
    -> *const f32;

    pub fn kunStreamPushData(context: KunStreamContextHandle, handle: size_t, buffer: *const f32);

    pub fn kunStreamRun(context: KunStreamContextHandle);
    pub fn kunDestoryStream(context: KunStreamContextHandle);
}
