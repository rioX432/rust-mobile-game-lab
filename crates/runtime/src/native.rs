#[cfg(any(target_vendor = "apple", target_os = "android"))]
use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

static RETAINED_TARGETS: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_os = "android")]
#[link(name = "log")]
unsafe extern "C" {
    fn __android_log_write(
        priority: i32,
        tag: *const std::ffi::c_char,
        text: *const std::ffi::c_char,
    ) -> i32;
}

pub fn trace(message: &str) {
    #[cfg(target_os = "android")]
    if let Ok(text) = std::ffi::CString::new(message) {
        // SAFETY: Both C strings are NUL terminated and live for this call.
        unsafe { __android_log_write(4, c"RustGameRuntime".as_ptr(), text.as_ptr()) };
    }
    #[cfg(not(target_os = "android"))]
    eprintln!("{message}");
}

pub struct NativeTarget {
    pub pointer: usize,
}

#[cfg(target_vendor = "apple")]
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_retain(value: *mut c_void) -> *mut c_void;
    fn objc_release(value: *mut c_void);
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
}

#[cfg(target_os = "android")]
#[link(name = "android")]
unsafe extern "C" {
    pub fn ANativeWindow_fromSurface(
        env: *mut jni::sys::JNIEnv,
        surface: jni::sys::jobject,
    ) -> *mut c_void;
    fn ANativeWindow_release(window: *mut c_void);
    fn ANativeWindow_acquire(window: *mut c_void);
}

impl NativeTarget {
    pub fn retained_count() -> usize {
        RETAINED_TARGETS.load(Ordering::Acquire)
    }

    #[cfg(target_os = "android")]
    pub fn from_acquired_window(pointer: *mut c_void) -> Self {
        RETAINED_TARGETS.fetch_add(1, Ordering::AcqRel);
        Self {
            pointer: pointer as usize,
        }
    }

    pub fn duplicate(&self) -> Self {
        #[cfg(target_vendor = "apple")]
        // SAFETY: self still owns a live retained layer.
        unsafe {
            objc_retain(self.pointer as *mut c_void)
        };
        #[cfg(target_os = "android")]
        // SAFETY: self still owns a live acquired native window reference.
        unsafe {
            ANativeWindow_acquire(self.pointer as *mut c_void)
        };
        RETAINED_TARGETS.fetch_add(1, Ordering::AcqRel);
        Self {
            pointer: self.pointer,
        }
    }

    /// # Safety
    /// `layer` must be a live CAMetalLayer for the duration of this call.
    #[cfg(target_vendor = "apple")]
    pub unsafe fn retain_layer(layer: *mut c_void) -> Self {
        RETAINED_TARGETS.fetch_add(1, Ordering::AcqRel);
        // SAFETY: The caller supplies a live Objective-C object. The retained
        // reference is released only after the owning Session's Surface.
        Self {
            pointer: unsafe { objc_retain(layer) } as usize,
        }
    }

    pub fn surface(&self, instance: &wgpu::Instance) -> Result<wgpu::Surface<'static>, String> {
        #[cfg(target_vendor = "apple")]
        let target = wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(self.pointer as *mut c_void);
        #[cfg(target_os = "android")]
        let target = {
            use raw_window_handle::{
                AndroidDisplayHandle, AndroidNdkWindowHandle, RawDisplayHandle, RawWindowHandle,
            };
            let pointer =
                std::ptr::NonNull::new(self.pointer as *mut c_void).ok_or("null native window")?;
            wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(RawDisplayHandle::Android(AndroidDisplayHandle::new())),
                raw_window_handle: RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(
                    pointer,
                )),
            }
        };
        #[cfg(any(target_vendor = "apple", target_os = "android"))]
        {
            // SAFETY: NativeTarget owns a native reference, and Session declares
            // its Renderer before NativeTarget so the Surface is dropped first.
            unsafe { instance.create_surface_unsafe(target) }.map_err(|e| e.to_string())
        }
        #[cfg(not(any(target_vendor = "apple", target_os = "android")))]
        {
            let _ = instance;
            Err("native embedding is supported on Android and Apple platforms".into())
        }
    }
}

impl Drop for NativeTarget {
    fn drop(&mut self) {
        #[cfg(target_vendor = "apple")]
        // SAFETY: This value owns one reference acquired with objc_retain.
        unsafe {
            objc_release(self.pointer as *mut c_void)
        };
        #[cfg(target_os = "android")]
        // SAFETY: ANativeWindow_fromSurface transferred one acquired reference.
        unsafe {
            ANativeWindow_release(self.pointer as *mut c_void)
        };
        RETAINED_TARGETS.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct AutoreleasePool {
    #[cfg(target_vendor = "apple")]
    pointer: *mut c_void,
}

impl AutoreleasePool {
    pub fn new() -> Self {
        Self {
            #[cfg(target_vendor = "apple")]
            // SAFETY: Each iteration creates and drains its pool on the worker.
            pointer: unsafe { objc_autoreleasePoolPush() },
        }
    }
}

impl Drop for AutoreleasePool {
    fn drop(&mut self) {
        #[cfg(target_vendor = "apple")]
        // SAFETY: This is the matching pool on the thread that created it.
        unsafe {
            objc_autoreleasePoolPop(self.pointer)
        };
    }
}
