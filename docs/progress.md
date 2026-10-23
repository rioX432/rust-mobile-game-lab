# 現在地（2026-10-24 JST）

rioが今回の到達範囲を「両OSでオフラインレースを一巡させ、検証・修正・PR・マージ・記録まで」と選択した。オフライン範囲の実装・両OS実行・独立レビューを確認済み。成果の受入と統合状態は[PR #15](https://github.com/rioX432/rust-mobile-game-lab/pull/15)に記録する。PoC全体は未完了。

## 作業ルートとPermissions

- https://github.com/rioX432/rust-mobile-game-lab （Public、rio承認済み）
- `/Users/rio/workspace/src/AnotherBall/rust-mobile-game-lab`、成果ブランチ`feat/offline-race`、統合先`main`。
- 実効設定は`workspace-write / on-request / auto_review`。rootはdeveloperの実効設定とpwd、独立WorktreeはCLIの`turn_context`で確認。CLI0.164.0の`--cd / --approve-for-me`を使用。`.codex/config.toml`への保存だけとは区別している。GUI表示は操作禁止のため未確認。
- Git作者はrioX432に紐付くrepo-local設定を維持。保護ルールを回避せず、merge commitのみを使う。
- 利用上限で止まった操作は未実行として保存し、rioの回復連絡後に正規の自動レビューで再開した。Full access/never等へ置換していない。

## 確認済みのオフライン成果

- RustのRoomがseed・入力・締切・勝敗の正本。両OSは同じFFI経路でStart/Choose/Closeを送信し、同じScene/Rendererで描画する。
- Rust core16 / renderer4 / runtime16テスト、format、strict Clippy成功。Android JUnit4件/lint/APK、iOS SDK26.5のSimulator実リンクビルド成功。
- 両OSそれぞれ、Race画面開閉10回、背景復帰5回、Probe/Race往復で同じRoomの保持、Resize、iOS New layer後のタッチ入力が成功。
- タッチのround1は両OSともseed42、順位[0,2,1]、人数[1,0,0]、勝者[2]。round2はnative Choose2、round3は再生成後タッチで結果まで進めた。
- 各回の管理インスタンス/worker/NativeTargetは各1、停止中の描画提出増分0、最終last_error/last_action_errorはnull。実提示FPSやGPUメモリの測定ではない。
- [操作・条件・画像・生記録](offline-native.md)。実画像を取得し目視確認した。Resultで背景復帰を検証しており、全phase総当たり・動的密度変更は未確認。

## Issue・PR

- 親: https://github.com/rioX432/rust-mobile-game-lab/issues/1 （PoC未完了、openを維持）
- コアレース: https://github.com/rioX432/rust-mobile-game-lab/issues/3 （コア受入条件の検証成功。受入の証拠は今回PRとrace.md）
- Android: https://github.com/rioX432/rust-mobile-game-lab/issues/6 、iOS: https://github.com/rioX432/rust-mobile-game-lab/issues/7 （オフライン埋め込み確認済み。密度変更/実機等の未確認は残す）
- 初期ネイティブPR: https://github.com/rioX432/rust-mobile-game-lab/pull/14 （merge済み、505c469）
- オフラインPR: https://github.com/rioX432/rust-mobile-game-lab/pull/15 。[Rust/Android/iOSのChecks](https://github.com/rioX432/rust-mobile-game-lab/pull/15/checks)で最新commitのCI結果を確認できる。Issue対応全件は[plan.md](plan.md)。

## レビューと実行証拠

- [初回FFI/Surface寿命レビュー](reviews/native-boundary.md): Metal取得待ち中のgate保持を修正し、遅延取得の故障注入テストと両OS再実行を確認済み。
- [オフライン規則/遅延入力/両OS境界レビュー](reviews/offline-race.md): 限定範囲でP1/P2なし。rootはその後に実アプリ検証を実施。
- repo固有clean-slop/create-prは存在しない。中央clean-slopを適用し、必要な安全契約コメントを保持した。PRはrepoテンプレートを使う。
- コード基点35a7354のアプリで実行。以後は文言一致だけの補助テストの除外と記録のみ。完全なローカルログは`artifacts/offline-{rust-validation,android-build,ios-build}.log`と`offline-mobile-acceptance.json`、iOS stderr/stdout、`offline-android-runtime.log`。両OSの実行ログでSurface破棄→Native参照解放の順を確認した。

## 保留した成果の保存先

今回のPRへ通信・SDKの途中実装は含めない。ソース/未コミット変更を削除・reset・stashしていない。

| ブランチ | 保存commit | 内容・未確認 |
|---|---|---|
| feat/race-network | 4e209e0 | 通信型e3daf2d、SDK生成/消費側・メモリ採取の実験。AAR消費側起動/タッチ、XCFramework生成と消費側ビルドまで。ゲーム/通信版SDKの最終確認とiOS消費側起動は未確認 |
| feat/room-server | 80f4875 | サーバー限定6テストとloopback実通信をworkerが確認。親レビュー/共通client E2E/モバイルは未確認 |
| feat/network-client | 1e736b1 | mock WebSocket peer/再接続等をworkerが検証。親レビュー/実サーバーE2E/wss実ハンドシェイク/モバイルは未確認 |
| feat/offline-native-race | 72c6685 | オフラインworker原本。main基点へcherry-pickして今回統合 |

各Worktreeとartifactsのタスク/結果を保持。SDK生成のXcode projectだけは`artifacts/deferred-sdk-generated/`へ移し、保留ブランチのソースと生成物は保持している。

## 未完了と次回

通信E2E/途中参加/再接続、投票、SDK最終取り込み、両OS実機、Release相当5分性能とサイズ差は未完了。動的密度変更、実Surface/Device Lost故障注入、モバイル初期化中の高速Closeも未確認。実機の対象指定は未確定で、利用していない。親Issueは閉じず、PoC完了としない。

次に進める場合はこのファイルとnetwork-contract（保留ブランチ）を読み、保留ブランチの実状態とPermissionsを確認してからサーバー/共通clientの統合E2Eを行う。重複Issueを作らない。今回の区切りではそこへ進めない。

再現はrepoルートで:

```sh
gh pr checks 15 --repo rioX432/rust-mobile-game-lab
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh run -p game-core --example offline_race --locked
scripts/android-build.sh
scripts/ios-build.sh simulator
```

専用端末はAndroid `RustMobileGameLab_API35_20261024` / emulator-5630とiOS Simulator `06E9B566-FCC7-4A9B-9A0D-046B9A28D126`。既存の無関係な端末は変更しない。起動後Open → Race → Start / next → 3秒以内にタッチ/Choose → Result、再度Start / nextで次ラウンド。
