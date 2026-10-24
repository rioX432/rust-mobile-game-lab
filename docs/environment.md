# 環境とPermissions | 2026-10-24 基準線
この内容は **旧オフライン検証時点の観測**。将来の実機/CI稼働環境を保証しない。最新のインストール/利用可能端末は各セッションで再確認する。

## 記録済み技術環境
- Rust toolchain固定 1.99.0（scripts/cargo.sh）、wgpu 30.0.1
- Android SDK35 / NDK 28.2.13676358 / AGP8.9.2 / Gradle8.11.1 / JDK17、arm64-v8a
- iOS Xcode26.6 / SDK26.5 / minimum iOS16 / arm64 Simulator と arm64 device build
- Androidの専用AVD: RustMobileGameLab_API35_20261024、iPhone17 iOS26.5専用Simulatorを以前作成。既存の無関係な端末を触らない。両者とも作業終了時に停止した記録あり。
- GitHub上のbaseline CIはRust/Android/iOS job成功: [PR #15](https://github.com/rioX432/rust-mobile-game-lab/pull/15)。3D/実機性能の証拠ではない。

## Codexの権限
`.codex/config.toml`で
```toml
sandbox_mode = "workspace-write"
approval_policy = "on-request"
approvals_reviewer = "auto_review"
```
を指定済み。ただし **設定ファイルだけではPermissions有効の証明にならない**。Codexの新しいセッションやWorktreeの作業前に実効cwd・権限を確認し、必要なら`--cd PATH --approve-for-me`を使う。Full access/neverの代替を使わず、許可されない端末や認証情報を触らない。Git作者の既存repo-local設定を維持する。

## 今後の測定
新しい3D方式/Android/iOS実機性能は[G03](https://github.com/rioX432/rust-mobile-game-lab/issues/19)、[G14](https://github.com/rioX432/rust-mobile-game-lab/issues/30)で計測。Emulator/Simulator・device build・実機実行・Release相当fps/熱は別ステータスにする。
