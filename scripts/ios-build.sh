#!/bin/bash
set -euo pipefail

repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"
mode=${1:-simulator}
case "$mode" in
    simulator|check-sources) sdk=iphonesimulator; target=aarch64-apple-ios-sim; triple=arm64-apple-ios16.0-simulator ;;
    device|check-device-sources) sdk=iphoneos; target=aarch64-apple-ios; triple=arm64-apple-ios16.0 ;;
    *) echo "Usage: $0 [simulator|device|check-sources|check-device-sources]" >&2; exit 2 ;;
esac

sdk_version=$(xcrun --sdk "$sdk" --show-sdk-version)
if [[ "$sdk_version" != "26.5" ]]; then
    echo "Expected $sdk SDK 26.5, found $sdk_version" >&2
    exit 1
fi
build_dir="$repo_dir/apps/ios/build"
mkdir -p "$build_dir"
if [[ "$mode" == check-* ]]; then
    sdk_path=$(xcrun --sdk "$sdk" --show-sdk-path)
    mkdir -p "$build_dir/source-check/$sdk"
    xcrun --sdk "$sdk" swiftc -emit-object -parse-as-library -whole-module-optimization \
        -swift-version 5 -target "$triple" -sdk "$sdk_path" \
        -module-name RustMobileLab -module-cache-path "$build_dir/ModuleCache" \
        -import-objc-header "$repo_dir/apps/ios/Sources/GameRuntime-Bridging-Header.h" \
        -I "$repo_dir/include" "$repo_dir"/apps/ios/Sources/*.swift \
        -o "$build_dir/source-check/$sdk/RustMobileLab.o"
    echo "Swift sources compiled for arm64 $sdk; Rust link NOT performed."
    exit 0
fi

export CARGO_TARGET_DIR="$repo_dir/target"
export IPHONEOS_DEPLOYMENT_TARGET=16.0
"$repo_dir/scripts/cargo.sh" build --locked --package game-runtime --lib --target "$target"
archive="$CARGO_TARGET_DIR/$target/debug/libgame_runtime.a"
symbols_file="$build_dir/$sdk-symbols.txt"
# Rust 1.99 bundles LLVM 23 bitcode; inspect native symbols with Xcode's older reader.
if ! xcrun nm --no-llvm-bc -gU "$archive" > "$symbols_file" 2> "$build_dir/$sdk-nm.log"; then
    head -20 "$build_dir/$sdk-nm.log" >&2
    exit 1
fi
for symbol in mg_create mg_destroy mg_attach_metal mg_resize mg_touch mg_pause mg_detach mg_action mg_status; do
    if ! /usr/bin/grep -Eq "[[:space:]]_${symbol}$" "$symbols_file"; then
        echo "Missing Rust symbol $symbol in $archive; integration required. App link NOT performed." >&2
        exit 1
    fi
done
mkdir -p "$build_dir/rust/$sdk"
cp "$archive" "$build_dir/rust/$sdk/libgame_runtime.a"
xcodegen generate --spec "$repo_dir/apps/ios/project.yml" --project "$repo_dir/apps/ios"
if [[ "$mode" == "simulator" ]]; then
    destination='generic/platform=iOS Simulator'
else
    destination='generic/platform=iOS'
fi
xcodebuild -project "$repo_dir/apps/ios/RustMobileLab.xcodeproj" -scheme RustMobileLab \
    -configuration Debug -sdk "$sdk" -destination "$destination" \
    -derivedDataPath "$build_dir/DerivedData-$sdk" \
    ARCHS=arm64 ONLY_ACTIVE_ARCH=NO CODE_SIGNING_ALLOWED=NO build
