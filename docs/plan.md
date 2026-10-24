# 現行バックログと依存 | SEED Race v1
更新: 2026-10-24。親Epic [#16](https://github.com/rioX432/rust-mobile-game-lab/issues/16)。**この表だけが現行Issueの並び順を定義する**。詳細はGitHub Issueを正本とし、本ファイルにIssue本文を複製しない。

| Priority | ID/Issue | 完了の中心 | 主な依存 |
| --- | --- | --- | --- |
| P0 | [G01 #17](https://github.com/rioX432/rust-mobile-game-lab/issues/17) | 旧計画と途中Worktreeの安全な移行 | - |
| P0 | [G02 #18](https://github.com/rioX432/rust-mobile-game-lab/issues/18) | Android/iOSの旧Native未確認条件 | G01 |
| P0 | [G03 #19](https://github.com/rioX432/rust-mobile-game-lab/issues/19) | 3D描画/物理方式の比較ADR | G01 |
| P0 | [G04 #20](https://github.com/rioX432/rust-mobile-game-lab/issues/20) | SEED参照・公開権利・2D/3D試作 | G01 |
| P1 | [G05 #21](https://github.com/rioX432/rust-mobile-game-lab/issues/21) | Creative-lab→ゲームContent Pack | G03,G04 |
| P0 | [G06 #22](https://github.com/rioX432/rust-mobile-game-lab/issues/22) | 20体の物理レース/Headless順位 | G03 |
| P0 | [G07 #23](https://github.com/rioX432/rust-mobile-game-lab/issues/23) | 最初のオフラインPlayable | G02,G03,G05,G06 |
| P1 | [G08 #24](https://github.com/rioX432/rust-mobile-game-lab/issues/24) | カメラ・配信者向けUI/名前/順位 | G07 |
| P1 | [G09 #25](https://github.com/rioX432/rust-mobile-game-lab/issues/25) | SEEDのモーション、FX、効果音 | G07,G04 |
| P1 | [G10 #26](https://github.com/rioX432/rust-mobile-game-lab/issues/26) | 権威的レースRoomサーバー | G06,G01 |
| P1 | [G11 #27](https://github.com/rioX432/rust-mobile-game-lab/issues/27) | Android/iOS参加と再接続 | G07,G10,G02 |
| P2 | [G12 #28](https://github.com/rioX432/rust-mobile-game-lab/issues/28) | server側Luaルール切替 | G10,G06 |
| P2 | [G13 #29](https://github.com/rioX432/rust-mobile-game-lab/issues/29) | AAR/XCFrameworkを別appで取り込む | G07,G11 |
| P1 | [G14 #30](https://github.com/rioX432/rust-mobile-game-lab/issues/30) | 実機/60fps目標/20・100体性能測定 | G07,G08,G09,G11,G13 |
| P2 | [G15 #31](https://github.com/rioX432/rust-mobile-game-lab/issues/31) | 最終ゲーム品質・再利用設計・導入判断 | G07〜G14の必須証拠 |

## 先に到達するマイルストーン
**M1: G07のオフラインPlayable**。SEED1体と独自の1コースで、物理に基づく10〜20秒のレースをAndroid/iOSで実際に動かす。G10のRoomサーバー基盤の独立実装は先行できるが、G07の完了判定をネットワーク待ちにしない。

**M2: G11の視聴者参加**。Android/iOSからルームへ参加し同じ結果を受け取り、途中入退出/再接続を確認する。

**M3: G15の作品品質とSDK検証**。アート/FX/音、Lua更新、Consumer SDK、実機測定を別ゲートとして合流する。

## 旧Issueからの移管
- [旧親 #1](https://github.com/rioX432/rust-mobile-game-lab/issues/1) は新Epic #16に置換。
- 旧 #2〜#5 はオフライン成功のClosed証拠として保持。
- 旧 #6/#7 の動的密度・高速Close・GPU/Surface喪失・各phase背景復帰などは **G02 (#18)**。
- 旧 #8/#9 のRoomサーバー/通信クライアントは **G10 (#26), G11 (#27)**。ローカル途中成果が記録されているが実体と適合確認前は未統合。
- 旧 #10 は **G11 (#27)** と **G14 (#30)**、旧 #11 の「2本目の3択投票」は今回の必須スコープから除外、旧 #12 は **G13 (#29)**、旧 #13 は **G14 (#30) / G15 (#31)**。
- 削除される旧`docs/issues`と旧`prompts/codex-poc.md`は**現行タスクの発見元ではない**。過去のIssue/PRはリンクで追う。

## AIの運用
別Worktree/共有契約統合担当の分離、Approve for me、受入の証拠、実装済みと未確認の区別は[AGENTS.md](../AGENTS.md)に従う。優先度順ではなく **依存が揃ったReady Issue** を並列実行してよい。CIで新しい文章ドリフトガードは作らない。
