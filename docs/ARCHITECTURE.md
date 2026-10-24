# SEED Race | Runtime・サーバー・制作基盤の設計
更新: 2026-10-24。これは採用済みと**未実装の提案**を明確に分けた設計であり、実装上の正本は各公開APIと動作するテスト。

## 既にある基準線
- `crates/core`: OS/GPUに依存しない3択オフラインレース状態遷移、`Scene`。
- `crates/renderer`: wgpuで矩形/円を表示する2D Renderer。
- `crates/runtime`: C ABI/JNI、世代付きSurface/instance、GPUワーカーと入力キュー。
- `apps/android`、`apps/ios`、`apps/desktop`: ホストのOS UI・描画領域。
- 実績: [PR #14](https://github.com/rioX432/rust-mobile-game-lab/pull/14)、[#15](https://github.com/rioX432/rust-mobile-game-lab/pull/15)。旧2Dを消さずに回帰可能にする。

## 次に構築する構造（未実装）
```text
Private creative-lab/Blender ----> editable source + inspected asset pack
                                     | (rights/licensing/sha gates)
                                     v
game asset importer --------------> SEED visuals / track / collider / FX / audio
                                     |
Android/iOS Native Host --> Rust client runtime + renderer + interpolation
                                     ^
                                     | versioned snapshots / events
                                     v
Rust authoritative room server --> fixed-step physics + standings
                                     ^
                                     | bounded typed commands at round boundaries
                                     v
Private native-script-update-lab runtime -> game-specific server Lua adapter
```
- ホストNativeはSurface/画面/権限/接続ライフサイクルを担当。ゲーム規則と結果はRust。
- サーバーで物理stepを進めて順位を確定し、クライアントは補間して表示。GPU描画結果から順位を決めない。異なるOS/GPU間の浮動小数点ビット完全一致は要求しない。
- 物理はRapier3Dを**比較候補**とする。固定step、単位系、Colliderと見た目の分離、同期するstate/snapshot設計はG03/G06で確定。
- 3D描画は既存wgpuの拡張と既成3D機能を持つエンジン(例: Bevy)をG03で実測比較。採用決定前は3D完成と扱わない。旧Renderer/FFIの寿命契約を維持する。
- 通信のroom/auth/revision/reconnect/重複コマンドはG10/G11の責務。保存済みローカルWorktreeの実装はレビューするまで未統合。
- LuaはG12以降、サーバーがイベント+snapshotを与え、Scriptが型付きコマンドを返し、検証後に適用する。進行中のroundでは`script_version`とdigestを固定。不正更新は旧版維持。Private SDKソースをPublic repoへ貼り付けない。
- `creative-lab`は制作/比較/出典保存を担当。ゲーム用のImporter/Content Pack/実再生QAはこのrepoが担当。`avvy-fx-factory`は企業専用の制作対象で、この個人PoCのruntime依存にしない。

## 境界変更のレビュー
変更する契約ごとに (1) 現行利用者 (2) 受入条件 (3) OS別ライフサイクル (4) サイズ/性能 (5) 互換性/rollback を明記。共有`Cargo.lock`・FFIヘッダー・Protocolは統合担当が所有。過剰な事前抽象化より、最初に1本のゲームが実際に動くことを優先する。

現行2Dの具体的C ABIは[contracts.md](contracts.md)、旧実測は[offline-native.md](offline-native.md)、意思決定は関連IssueのADRに残す。
