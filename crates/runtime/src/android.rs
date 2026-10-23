use super::*;
use jni::{
    JNIEnv,
    objects::{JClass, JObject},
    sys::{jboolean, jfloat, jint, jlong, jstring},
};

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_create(
    _env: JNIEnv,
    _class: JClass,
    seed: jlong,
) -> jlong {
    mg_create(seed as u64) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_destroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    mg_destroy(handle as u64)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_attach(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    surface: JObject,
    width: jint,
    height: jint,
    scale: jfloat,
) -> jint {
    guarded(|| {
        if surface.is_null() || width < 0 || height < 0 {
            return Err(INVALID_ARGUMENT);
        }
        control(handle as u64)?;
        // SAFETY: JNI provides a live Surface local reference and thread-local
        // JNIEnv during this call. The NDK function acquires a native reference.
        let pointer = unsafe {
            native::ANativeWindow_fromSurface(env.get_native_interface(), surface.as_raw())
        };
        if pointer.is_null() {
            return Err(INVALID_ARGUMENT);
        }
        attach(
            handle as u64,
            NativeTarget::from_acquired_window(pointer),
            width as u32,
            height as u32,
            scale,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_resize(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    width: jint,
    height: jint,
    scale: jfloat,
) -> jint {
    if width < 0 || height < 0 {
        return INVALID_ARGUMENT;
    }
    mg_resize(handle as u64, width as u32, height as u32, scale)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_touch(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jfloat,
    y: jfloat,
) -> jint {
    mg_touch(handle as u64, x, y)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_pause(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    paused: jboolean,
) -> jint {
    mg_pause(handle as u64, u32::from(paused != 0))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_detach(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    mg_detach(handle as u64)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_action(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    action: jint,
    argument: jint,
) -> jint {
    mg_action(handle as u64, action as u32, argument as u32)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_lab_rustmobile_NativeBridge_status(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    catch_unwind(AssertUnwindSafe(|| {
        let value =
            status_json(handle as u64).unwrap_or_else(|error| format!("{{\"error\":{error}}}"));
        env.new_string(value)
            .map_or(std::ptr::null_mut(), |value| value.into_raw())
    }))
    .unwrap_or(std::ptr::null_mut())
}
