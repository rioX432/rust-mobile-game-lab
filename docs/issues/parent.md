Rust共通の2Dゲーム・描画・通信基盤をAndroid/iOSのネイティブ画面内へ埋め込むPoC。

仕様全文: prompts/codex-poc.md。公開への変更はrio承認済み。

最初に図形・タッチ・Surface再生成を両OSへ縦につなぎ、その後レース→通信→投票/SDK→実機測定へ進む。

必須条件が未検証の間はこの親Issueを閉じない。専用作業ルートと実効Approve for meを確認し、共通Rust描画器のAndroid/iOSネイティブUI内でのビルド・Emulator/Simulator起動・入力・開閉10回・背景復帰5回・再生成を確認した。独立レビューのMetal取得待ちP1を修正し、限定再レビューと両OS再検証が成功。実機・性能、ゲーム/通信/投票/SDK消費側は未確認。

## 依存する子Issue

- [ ] #2 開発環境・共通契約・CIの最小構成（依存: なし）
- [ ] #3 Rustコアとミニレースの状態遷移（依存: #2）
- [ ] #4 wgpu描画器とDesktop検証シェル（依存: #2）
- [ ] #5 FFI・ランタイム・Surfaceの寿命管理（依存: #2、#4）
- [ ] #6 Androidへの部分埋め込み（依存: #5）
- [ ] #7 iOSへの部分埋め込み（依存: #5）
- [ ] #8 通信プロトコルとルームサーバー（依存: #3）
- [ ] #9 共通通信クライアントと再接続（依存: #5、#8）
- [ ] #10 Android・iOSを含む参加型E2Eと復帰検証（依存: #6、#7、#9）
- [ ] #11 3択投票を追加して再利用性を検証（依存: #4、#9）
- [ ] #12 AAR／XCFrameworkと取り込みサンプル（依存: #6、#7、#9）
- [ ] #13 性能測定・最終検証・導入判断（依存: #10、#11、#12）

## 必須条件と未確認事項

- [x] 専用作業ルートとApprove for meの現セッションへの反映
- [x] 両OSの部分描画/入力と開閉10回/復帰5回/再生成（Emulator/Simulator。実機成功とは扱わない）
- [ ] 2クライアント実通信、途中参加、再接続、約20合成参加者
- [ ] 両ゲームを両OS実機で動作確認
- [ ] AAR/XCFramework消費側のビルド・起動
- [ ] Release相当の両OS実機性能とサイズ差分

Android実機接続なし。iOS実機は今回の使用対象として未確認。Emulator/SimulatorやDesktopの成功で実機条件を置換しない。

詳細と実画像: docs/native-verification.md、docs/evidence/native-mobile-smoke.json、docs/reviews/native-boundary.md。実Surface/Device Lostの故障注入、モバイル初期化中の高速Closeは未確認。Rustコアのレースは別Worktreeで実装・テスト済み、統合と実ホスト接続へ進む。全Issueは受入条件が揃ってから閉じる。
