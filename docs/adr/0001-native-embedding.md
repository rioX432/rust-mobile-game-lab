# Rust + wgpu + thin native hosts

Rust 1.99.0を固定し、wgpu 30.0.1、winit 0.30.13を採用する。wgpuの公式資料でMetal/VulkanとMSRV 1.87を確認した。実際のAPIは固定版の依存ソースとも照合する。ゲームエンジン全体は追加せず、描画データだけを共有する。

AndroidはSurfaceView/JNIとNDKのANativeWindow、iOSはUIView/CAMetalLayer/C ABI。ネイティブ参照をRustが保持し、GPU Surfaceより後に解放する。初期化はRustワーカーで行う。Desktopは同じRendererにwinitのWindowを渡す開発用シェル。

AndroidはAGP 8.9.2 / Gradle 8.11.1 / JDK 17 / compileSdk 35 / minSdk 26 / NDK 28.2.13676358を基準とする。NDKは端末ページサイズへの対応を含め公式資料とビルド実行で検証する。iOSはdeployment target 16、arm64実機・arm64 Simulatorを初期対象とする。Intel Simulatorは未対応として明記し、使用可能なら別途追加する。

参考（2026-10-24確認）:
- https://docs.rs/wgpu/30.0.1/wgpu/
- https://docs.rs/wgpu/30.0.1/x86_64-apple-darwin/wgpu/enum.SurfaceTargetUnsafe.html
- https://docs.rs/winit/0.30.13/winit/
- https://developer.android.com/ndk/reference/group/a-native-window
- https://developer.android.com/build/releases/agp-8-9-0-release-notes
- https://rust-lang.github.io/rustup/cross-compilation.html
