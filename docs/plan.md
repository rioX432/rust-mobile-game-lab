# 実装計画

仕様の正本: [codex-poc.md](../prompts/codex-poc.md)。親Issue: https://github.com/rioX432/rust-mobile-game-lab/issues/1

| 作業 | Issue | 依存 |
|---|---|---|
| 開発環境・共通契約・CIの最小構成 | [P01 #2](https://github.com/rioX432/rust-mobile-game-lab/issues/2) | なし |
| Rustコアとミニレースの状態遷移 | [P02 #3](https://github.com/rioX432/rust-mobile-game-lab/issues/3) | #2 |
| wgpu描画器とDesktop検証シェル | [P03 #4](https://github.com/rioX432/rust-mobile-game-lab/issues/4) | #2 |
| FFI・ランタイム・Surfaceの寿命管理 | [P04 #5](https://github.com/rioX432/rust-mobile-game-lab/issues/5) | #2、#4 |
| Androidへの部分埋め込み | [P05 #6](https://github.com/rioX432/rust-mobile-game-lab/issues/6) | #5 |
| iOSへの部分埋め込み | [P06 #7](https://github.com/rioX432/rust-mobile-game-lab/issues/7) | #5 |
| 通信プロトコルとルームサーバー | [P07 #8](https://github.com/rioX432/rust-mobile-game-lab/issues/8) | #3 |
| 共通通信クライアントと再接続 | [P08 #9](https://github.com/rioX432/rust-mobile-game-lab/issues/9) | #5、#8 |
| Android・iOSを含む参加型E2Eと復帰検証 | [P09 #10](https://github.com/rioX432/rust-mobile-game-lab/issues/10) | #6、#7、#9 |
| 3択投票を追加して再利用性を検証 | [P10 #11](https://github.com/rioX432/rust-mobile-game-lab/issues/11) | #4、#9 |
| AAR／XCFrameworkと取り込みサンプル | [P11 #12](https://github.com/rioX432/rust-mobile-game-lab/issues/12) | #6、#7、#9 |
| 性能測定・最終検証・導入判断 | [P12 #13](https://github.com/rioX432/rust-mobile-game-lab/issues/13) | #10、#11、#12 |

最初に開発環境/Permissionsと最小契約を確定し、図形・タッチ・Surface再生成をAndroid/iOSで確認する。ゲーム完成を待たない。次にオフラインレース、通信、再接続、投票、SDK消費、実機性能の順で接続する。

共有契約確定後、独立したAndroid/iOS/サーバー作業を別Worktreeのサブエージェントへ分担する。同じディレクトリを共同編集しない。共有契約とlockfileは統合担当が所有する。

重点レビュー: ネイティブ/FFI寿命 → 通信/復旧 → 両ゲーム/SDK/最終受入。重大指摘を修正し、各PRに必要な検証とレビューを通してからmerge commitでmainへ統合する。受入未検証のIssueには自動closeキーワードを使わない。
