#!/bin/sh
set -eu
repo_dir=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"

host_only=false
case "${1:-}" in
    --host-only) host_only=true; shift ;;
    "") ;;
    *) echo "Usage: scripts/android-build.sh [--host-only]" >&2; exit 2 ;;
esac
[ "$#" -eq 0 ] || { echo "Unexpected arguments" >&2; exit 2; }

case "$(uname -s)" in
    Darwin)
        sdk_default="$HOME/Library/Android/sdk"
        ndk_host=darwin-x86_64
        if [ -z "${JAVA_HOME:-}" ]; then JAVA_HOME=$(/usr/libexec/java_home -v 17); fi
        ;;
    Linux) sdk_default="$HOME/Android/Sdk"; ndk_host=linux-x86_64 ;;
    *) echo "Supported build hosts: macOS and Linux" >&2; exit 2 ;;
esac
if [ -n "${JAVA_HOME:-}" ]; then export JAVA_HOME; export PATH="$JAVA_HOME/bin:$PATH"; fi
java_version=$(java -version 2>&1)
case "$java_version" in
    *'version "17.'*) ;;
    *) echo "JDK 17 is required; set JAVA_HOME to a JDK 17 installation" >&2; exit 2 ;;
esac
export ANDROID_HOME="${ANDROID_HOME:-$sdk_default}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/28.2.13676358}"
[ -f "$ANDROID_HOME/platforms/android-35/android.jar" ] || {
    echo "Install Android SDK platform 35 in ANDROID_HOME=$ANDROID_HOME" >&2; exit 2;
}

if [ "$host_only" = true ]; then
    exec "$repo_dir/apps/android/gradlew" -p "$repo_dir/apps/android" --no-daemon --console=plain \
        :app:compileDebugKotlin :app:testDebugUnitTest :app:lintDebug
fi

if ! grep -Eq '^Pkg.Revision = 28\.2\.13676358$' "$ANDROID_NDK_HOME/source.properties"; then
    echo "NDK 28.2.13676358 is required; set ANDROID_NDK_HOME" >&2
    exit 2
fi
ndk_bin="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$ndk_host/bin"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ndk_bin/aarch64-linux-android26-clang"
export CC_aarch64_linux_android="$CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"
export AR_aarch64_linux_android="$ndk_bin/llvm-ar"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_dir/target}"
case "$CARGO_TARGET_DIR" in /*) ;; *) export CARGO_TARGET_DIR="$repo_dir/$CARGO_TARGET_DIR" ;; esac
"$repo_dir/scripts/cargo.sh" build --locked --release -p game-runtime --target aarch64-linux-android
library="$CARGO_TARGET_DIR/aarch64-linux-android/release/libgame_runtime.so"
symbols=$("$ndk_bin/llvm-nm" --dynamic --defined-only "$library")
for method in create destroy attach resize touch pause detach action status; do
    if ! printf '%s\n' "$symbols" | grep -Eq "[[:space:]]Java_lab_rustmobile_NativeBridge_${method}$"; then
        echo "Missing JNI export: Java_lab_rustmobile_NativeBridge_$method; integrate the real runtime before APK build" >&2
        exit 1
    fi
done
jni_dir="$repo_dir/apps/android/app/src/main/jniLibs/arm64-v8a"
mkdir -p "$jni_dir"
cp "$library" "$jni_dir/libgame_runtime.so"
"$repo_dir/apps/android/gradlew" -p "$repo_dir/apps/android" --no-daemon --console=plain \
    :app:testDebugUnitTest :app:lintDebug :app:assembleDebug
printf 'APK: %s\n' "$repo_dir/apps/android/app/build/outputs/apk/debug/app-debug.apk"
