# 現行バックログと依存 | SEED Race v1
更新: 2026-10-24。親Epic [#16](https://github.com/rioX432/rust-mobile-game-lab/issues/16)。**この表だけが現行Issueの並び順を定義する**。詳細はGitHub Issueを正本とし、本ファイルにIssue本文を複製しない。

| Priority | ID/Issue | 完了の中心 | 主な依存 |
| --- | --- | --- | --- |
| P0 | [G01 #17](https://github.com/rioX432/rust-mobile-game-lab/issues/17) | 旧計画と途中Worktreeの安全な移行 | - |
| P0 | [G02 #18](https://github.com/rioX432/rust-mobile-game-lab/issues/18) | Android/iOSの旧Native未確認条件 | G01 |
| P0 | [G03 #19](https://github.com/rioX432/rust-mobile-game-lab/issues/19) | 3D描画/物理方式の比較ADR（Host-owned GPU適性も評価） | G01 |
| P0 | [G19 #37](https://github.com/rioX432/rust-mobile-game-lab/issues/37) | Host-owned GPU/Embedded描画・dummy合成API。2D先行 | G01、G03を参照 |
| P0 | [G04 #20](https://github.com/rioX432/rust-mobile-game-lab/issues/20) | SEED参照・公開権利・2D/3D試作 | G01 |
| P1 | [G05 #21](https://github.com/rioX432/rust-mobile-game-lab/issues/21) | Creative-lab→ゲームContent Pack | G03,G04 |
| P0 | [G06 #22](https://github.com/rioX432/rust-mobile-game-lab/issues/22) | 20体の物理レース/Headless順位 | G03 |
| P1 | [G16 #33](https://github.com/rioX432/rust-mobile-game-lab/issues/33) | 基準SEED1体の編集可能な完成アセット | G04,G05の最小Export契約（G03の選定を参照） |
| P1 | [G17 #34](https://github.com/rioX432/rust-mobile-game-lab/issues/34) | 独自コース・障害物・背景の実制作 | G03,G05の最小契約,G06のCollider契約 |
| P0 | [G07 #23](https://github.com/rioX432/rust-mobile-game-lab/issues/23) | 最初のオフラインPlayable | G02,G03,G05,G06,G16,G17 |
| P1 | [G08 #24](https://github.com/rioX432/rust-mobile-game-lab/issues/24) | カメラ・配信者向けUI/名前/順位 | G07 |
| P1 | [G09 #25](https://github.com/rioX432/rust-mobile-game-lab/issues/25) | SEEDのモーション、FX、効果音 | G07,G04（G16のキャラ正本を使用） |
| P1 | [G18 #35](https://github.com/rioX432/rust-mobile-game-lab/issues/35) | 実ゲーム動画からAI批評→局所修正→再実行を2サイクル | G07,G16,G17（G09の演出も比較対象） |
| P1 | [G10 #26](https://github.com/rioX432/rust-mobile-game-lab/issues/26) | 権威的レースRoomサーバー | G06,G01 |
| P1 | [G11 #27](https://github.com/rioX432/rust-mobile-game-lab/issues/27) | Android/iOS参加と再接続 | G07,G10,G02 |
| P2 | [G12 #28](https://github.com/rioX432/rust-mobile-game-lab/issues/28) | server側Luaルール切替 | G10,G06 |
| P2 | [G13 #29](https://github.com/rioX432/rust-mobile-game-lab/issues/29) | Standalone AAR/XCFrameworkを別appで取り込む | G07,G11 |
| P1 | [G20 #38](https://github.com/rioX432/rust-mobile-game-lab/issues/38) | Private配信HostでAvatar・RTCとゲームを同時実行 | G19,G11,G13 |
| P1 | [G14 #30](https://github.com/rioX432/rust-mobile-game-lab/issues/30) | 実機/60fps目標/20・100体性能測定 | G07,G08,G09,G11,G13 |
| P2 | [G15 #31](https://github.com/rioX432/rust-mobile-game-lab/issues/31) | 最終ゲーム品質・再利用設計・導入判断 | G07〜G14とG16〜G18の必須証拠 |

## 先に到達するマイルストーン
**M1: G07のオフラインPlayable**。G16のSEED基準キャラ1体とG17の独自コース1つを取り込み、物理に基づく10〜20秒のレースをAndroid/iOSで実際に動かす。残り9体の制作やG18の反復改善をM1完了の前提にしない。G10のRoomサーバー基盤の独立実装は先行できるが、G07の完了判定をネットワーク待ちにしない。

**M2: G11の視聴者参加**。Android/iOSからルームへ参加し同じ結果を受け取り、途中入退出/再接続を確認する。

**M3: G15の作品品質とSDK検証**。G16/G17の制作正本とG18の実ゲーム動画に基づく反復改善、G09のモーション/FX/音、Lua更新、Consumer SDK、実機測定を別ゲートとして合流する。

**M4: 組み込み互換性（SEED Race本編の完成とは独立）**。G19でPublicのdummy Hostに共有GPU描画/合成、G20でPrivate ConsumerにAvatar/RTCとゲームを同居させる。G13 standalone SDK成功をEmbedded成功と混同しない。未受入でもG07/G15のゲーム制作ゲートを止めない。[設計](EMBEDDED_RENDERER.md)。

## アセット制作Issueの責務（再計画2026-10-24）
- G04 #20: 原画に忠実な**仕様/権利確認と2D・3D比較**。正式なキャラの量産ではない。
- G05 #21: **制作→Content Pack→ゲーム読み込みの契約**。本番の芸術作品を代わりに量産するのではない。
- G16 #33: **編集可能なSEED1体のアート制作**。G07で利用。残り9体は後続。
- G17 #34: **独自コース1つ＋主要障害物/背景のアート制作**。G06物理とのCollider境界を連携。
- G09 #25: 制作済み正本へ**動き・FX・音**を付ける。
- G18 #35: G07以後の実際のゲームフレームで**2サイクル以上のAI批評/局所修正/比較**を行う。ユーザーの美術最終採用を自動で代行しない。
- G07 #23はG16/G17の実アセットを取り込むが、G18を待たずにプレイアブルとして受け入れる。G15 #31はG18の比較結果を最終品質へ引き継ぐ。

## 旧Issueからの移管
- [旧親 #1](https://github.com/rioX432/rust-mobile-game-lab/issues/1) は新Epic #16に置換。
- 旧 #2〜#5 はオフライン成功のClosed証拠として保持。
- 旧 #6/#7 の動的密度・高速Close・GPU/Surface喪失・各phase背景復帰などは **G02 (#18)**。
- 旧 #8/#9 のRoomサーバー/通信クライアントは **G10 (#26), G11 (#27)**。ローカル途中成果が記録されているが実体と適合確認前は未統合。
- 旧 #10 は **G11 (#27)** と **G14 (#30)**、旧 #11 の「2本目の3択投票」は今回の必須スコープから除外、旧 #12 は **G13 (#29)**、旧 #13 は **G14 (#30) / G15 (#31)**。
- 削除される旧`docs/issues`と旧`prompts/codex-poc.md`は**現行タスクの発見元ではない**。過去のIssue/PRはリンクで追う。

## AIの運用
別Worktree/共有契約統合担当の分離、Approve for me、受入の証拠、実装済みと未確認の区別は[AGENTS.md](../AGENTS.md)に従う。優先度順ではなく **依存が揃ったReady Issue** を並列実行してよい。CIで新しい文章ドリフトガードは作らない。
