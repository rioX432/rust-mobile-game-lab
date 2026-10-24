# iOS | 現行起動・検証ガイド (SEED Race v1)
更新: 2026-10-24。現行の**2D/3択オフライン技術baseline**の手順。新しいSEED 3D物理レース・オンライン参加は未実装。

## ビルド
Xcode、XcodeGen、Rust iOS Simulator targetを使用する。既存の `scripts/ios-build.sh` は既定のSDK 26.5を要求する。異なるSDKの場合は根拠とともに`IOS_SDK_VERSION`を指定する。

```sh
scripts/ios-build.sh check-sources          # Swiftだけのコンパイル（Rustとのリンクなし）
scripts/ios-build.sh check-device-sources
scripts/ios-build.sh simulator              # Rust staticlib生成・ABI確認・アプリのリンク
scripts/ios-build.sh device                 # arm64 device用に署名なしでビルド
```

生成物は `apps/ios/build/DerivedData-iphonesimulator/Build/Products/Debug-iphonesimulator/RustMobileLab.app` を基本とし、実際のSDK/build script出力で確認する。自動インストール・サイン・実機起動はbuild scriptの担当外。

専用Simulatorに手動で入れてOpen → Race → Start / next → 3秒以内のChoose/タッチ → Result → 次ラウンド。Resize / New layerではCAMetalLayerのdetach/attachを伴うため、入力・解放順を確認する。実際の操作と状態は[オフライン証拠](offline-native.md)を参照。

## 残件
iPhone17 iOS26.5 Simulatorでは一巡、開閉10回、Resultから背景復帰5回、Layer再生成後のタッチを確認。実機Release、動的密度、全phase復帰、Device Lost試験は未確認。G02 #18とG14 #30で対応。既存FFIの所有権とMetalの取得gate/timeoutを壊さない。
