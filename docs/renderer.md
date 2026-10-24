# Renderer | 現行2Dの仕様と3Dへの拡張判断
更新: 2026-10-24。

## 実装済みの基準線
`crates/renderer` はwgpuにより`game_core::Scene`の円と矩形を描画する。Desktopは`apps/desktop`のwinitシェル。数値・RGBA・論理座標を検査し、リサイズ/ゼロサイズ/取得失効・一時停止からの再開を扱う。描画器は勝敗を変更しない。Native HostでのSurface所有・停止・Metal制約は[contracts](contracts.md)と[旧レビュー](reviews/native-boundary.md)。

```sh
scripts/cargo.sh test -p game-renderer -p game-core --locked
scripts/cargo.sh run -p game-desktop -- --smoke
scripts/cargo.sh clippy -p game-renderer -p game-desktop --all-targets --locked -- -D warnings
```

PR #14ではmacOS実GPUのDesktop smokeと、両OSネイティブ埋め込みを確認。PR #15では同じRendererでオフライン3択を表示した。ソフトウェア描画/デスクトップ提出回数は実機FPSの証拠ではない。

## 2D Embedded API（GPUはHostが所有）

- `game_renderer::EmbeddedRenderer` は現行2D描画を外部のDevice/Queueで動かすための追加入口。Standaloneの `Renderer` はそのまま維持する。
- `new(host_device, host_queue, target_format)` でGPUを借り、`encode(scene, texture_view, command_encoder, load_op)` でHostのEncoderにpassを書き込む。**GPU Device生成、Surface configure、Queue submit、presentを行わない**。
- 同一Rustリンク単位の呼出し専用。2Dは同じshape shader/pipelineを使う。Offscreen合成時の透過クリア、color/alpha、単一Present、Device loss、3Dへの拡張は [Embedded設計](EMBEDDED_RENDERER.md) と G19 #37の後続受入。
- GPU smoke testは `#[ignore]` のためGPU搭載環境で明示実行が必要。CIでRustコードがコンパイルできても実アバター合成の成功ではない。

## 3Dは未選定
SEED Raceの3D技術候補は「既存wgpuへの小さな3D拡張」と「Bevy等のRenderer統合」。[G03 #19](https://github.com/rioX432/rust-mobile-game-lab/issues/19)で同じ試験シーンをDesktop・Android/iOS Nativeに埋め込み、品質/性能/保守性を比較してADRを作成する。勝手にRendererを置換しない。3D案が採用される前に、既存2D/Native寿命機能を削除しない。
