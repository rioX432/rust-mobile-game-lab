# Android部分埋め込みホスト（Issue #6 / 環境 Issue #2）

- 担当Issue: https://github.com/rioX432/rust-mobile-game-lab/issues/6
- 環境Issue: https://github.com/rioX432/rust-mobile-game-lab/issues/2
- 基点: `feat/native-embedding` の `5fef344da0bf218bf1dfdf807f6399b94c007cb8`
- 作業ブランチ: `feat/android-native`
- 変更範囲: `apps/android/**`、`scripts/android*.sh`、本書。共有runtime/JNI、Cargo設定とlock、共有契約は変更しない。
- 権限: セッション `01a16982-8738-7c00-be76-bba44a8b0675` の `thread_settings_applied` と `pwd` から当該Worktree、`on-request`、`auto_review`、Worktree/一時領域を書込み可能とする制限付きworkspace profileを確認。証拠は `artifacts/worker-permissions.txt`。GUI選択は未確認。

## 固定環境と出典

| 項目 | 固定値 |
| --- | --- |
| Android Gradle Plugin | 8.9.2 |
| Gradle | 8.11.1 |
| JDK / JVM target | 17 |
| Kotlin Gradle Plugin / Kotlin | 2.2.10 |
| compileSdk / targetSdk / minSdk | 35 / 35 / 26 |
| Build Tools | 35.0.0 |
| NDK | 28.2.13676358 |
| ABI / Rust target | arm64-v8a / aarch64-linux-android |

2026-10-24 JSTに公式資料を照合した。

- [AGP 8.9の互換表と8.9.2修正](https://developer.android.com/build/releases/agp-8-9-0-release-notes): Gradle 8.11.1、JDK 17、最大API 35。NDKは指定版を使用する。
- [Kotlin互換表](https://kotlinlang.org/docs/gradle-configure-project.html): 2.2.0–2.2.10はGradle 7.6.3–8.14、AGP 7.3.1–8.10.0をサポートし、この構成を含む。
- [NDK linkerの指定方法](https://developer.android.com/ndk/guides/other_build_systems): NDK Clangのtarget/API suffixを用いる。Apple SiliconでもNDK r28の配布ディレクトリ名は `darwin-x86_64`。
- [SurfaceHolder.Callback](https://developer.android.com/reference/android/view/SurfaceHolder.Callback): `surfaceDestroyed`から戻る前に当該Surfaceへの描画を止める。API 35の導入済み `android.jar` でもcallback、Surface、MotionEventの署名を `javap` で照合。
- [Native Window](https://developer.android.com/ndk/reference/group/a-native-window): ANativeWindowの取得/保持/解放はRust側の責務。

### Gradle wrapperの取得とチェックサム

[公式Gradle 8.11.1 binary distribution](https://downloads.gradle.org/distributions/gradle-8.11.1-bin.zip)を取得し、[公式SHA-256一覧](https://gradle.org/release-checksums/)と照合してから、その配布物の `gradle wrapper` で生成した。既存persona由来のwrapperは使用していない。

| 対象 | SHA-256 |
| --- | --- |
| gradle-8.11.1-bin.zip | `f397b287023acdba1e9f6fc5ea72d22dd63669d59ed4a289a29b1a76eee151c6` |
| gradle-wrapper.jar | `2db75c40782f5e8ba1fc278a5574bab070adccb2d21ca5a6e5ed840888448046` |
| gradlew | `a3648413b47ef77af21d5ebc36c687c7d103aaef3e17f33de7d4f080a6f300a3` |
| gradlew.bat | `57931b17dd228e5c24dac90e815d0bf82477e831a4618dfab4136f5446b42a9f` |

distributionのSHAはwrapper設定にも固定。生成ログは `artifacts/android-wrapper.log`。Gradleのローカル通信がsandboxで拒否されたため、正規の `require_escalated` で生成・ビルドを実行する。信頼設定やユーザー設定は変更しない。

## ビルド

リポジトリrootから実行する。JDK 17、SDK Platform 35、Build Tools 35.0.0、NDK 28.2.13676358、root指定のRust 1.99.0とAndroid targetが必要。CIも同じスクリプトを呼べる。

```sh
export JAVA_HOME=/path/to/jdk17
export ANDROID_HOME=/path/to/android-sdk
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
rustup target add --toolchain 1.99.0 aarch64-linux-android
scripts/android-build.sh
```

macOSは `JAVA_HOME` 未指定時に `java_home -v 17`、SDK未指定時に `~/Library/Android/sdk` を使う。LinuxのSDK既定は `~/Android/Sdk`。環境変数で上書き可能。スクリプトはNDK linkerと `scripts/cargo.sh build --locked --release` を使用し、HomebrewのRustへ切り替えない。実際の `libgame_runtime.so` の9個のJNI exportを検査してから `app/src/main/jniLibs/arm64-v8a/` へ配置し、単体テスト・lint・Debug APK生成を行う。APKは `apps/android/app/build/outputs/apk/debug/app-debug.apk`。RustライブラリはRelease最適化、ホストAPKはDebugであり、実機性能の証拠にはしない。

Rust/JNI未統合時にホストだけを検査するには以下を使う。APKは生成せず、ネイティブの代替ライブラリも作らない。

```sh
scripts/android-build.sh --host-only
```

この経路はKotlinコンパイル、純粋なホスト制御の単体テスト、Android lintまで。JNI未実装を実行済みと見なさない。生成ライブラリ、APK、Gradle/Kotlinキャッシュは追跡対象外。

## 所有・呼出し順と統合条件

- Activityは1画面につき1個の `ProbeSession` と `SurfaceView` を所有する。JNI呼出し、Surface callback、statusの250msポーリングはUIスレッドへ限定する。GPUの初期化/描画/破棄はRustワーカーが所有し、JNI呼出しはGPU初期化を待たずに戻ること。
- `NativeBridge` は契約どおりの `@JvmStatic external`。ライブラリ名は `game_runtime`、クラスは `lab.rustmobile.NativeBridge`。JNI本体は統合担当。標準の `Java_lab_rustmobile_NativeBridge_{create,destroy,attach,resize,touch,pause,detach,action,status}` exportをスクリプトが確認する。
- Open → create → pause → Surface作成/サイズ通知 → attach → 現在のpause状態を通知。初期ゼロサイズではattachしない。接続済みのゼロサイズはresizeへ渡し、Rust側で描画を止める。
- `attach`が返る前にRustがANativeWindowを取得保持し、その後のGPU初期化を非同期で行う。KotlinはSurfaceをretain/releaseしない。Rust Surfaceの破棄後にRustがANativeWindowを解放する。
- `surfaceDestroyed`ではdetachを呼び、接続を無効化する。Close/戻る/Activity破棄はdetach → destroy → callback除去 → view除去。detach/destroyの負の結果は安全な解放として扱わず明示的に失敗させる。Rustのteardownはキュー満杯時も当該Surfaceへの新しい描画を同期的に止め、遅延初期化の完了をUIで待たせない必要がある。
- 古い画面callbackはセッションとviewの同一性で遮断する。閉じたセッションは全操作を無視する。Surface生成/破棄とcloseで世代を進め、古い世代のstatusを採用しない。ポーリングはpause/closeで解除する。Rust側でも接続世代とハンドルの有効性確認が必要。
- onPauseでpause、onResumeでresume。巨大な復帰deltaを入れない処理はRust側の契約。サイズ・密度は `surfaceChanged` ごとに物理ピクセル寸法と現在のdensityを渡す。通常のActivity再生成時は古いハンドルを破棄する。
- タッチはSurface内のタップ完了座標のみ。アクセシビリティのclickは領域中央へ渡す。上下UIのResetは `action(handle, 0, 0)`、開閉は寿命管理。ゲームルール・レース・通信は接続しない。

## 親担当が実行するモバイル受入

本ワーカーは既存Emulator/実機を操作しない。以下は統合後に親担当が選んだ専用環境だけで行う。端末の指定なしにADBコマンドを実行しない。

1. 実JNIを統合して `scripts/android-build.sh`。対象ABIの専用端末へAPKをインストールし `lab.rustmobile/.MainActivity` を起動する。
2. Open後、上下UIと図形が共存する画像、TextViewの生status JSON、`RustMobileHost`/Rustログを保存する。SurfaceタッチとResetで別々にカウンタ/選択が変わることを観察する。
3. Close/Openを10回。Openカウンタは操作補助であり合格判定ではない。detach→destroy、閉じた後の残存描画なし、再Open時の `live_instances` が1から増えないことをログ/JSONで確認する。
4. 開いたままHome→復帰を5回。Returnsカウンタとpause/resume、描画復旧、過剰な時間更新がないことを確認する。
5. Resizeボタンで部分領域を変更し、画面回転/Surface再生成を確認する。密度変更は専用検証環境でのみ行い、元へ戻す。古いstatus/描画/入力が新画面へ作用しないことを確認する。
6. Emulator、Android実機、実機性能を分けて記録する。オフラインレースと通信は後続統合であり、Issue #6の全受入はまだ閉じない。

## 検証記録

2026-10-24 JST、当該Worktreeで確認:

| 段階 | 結果・証拠 |
| --- | --- |
| Permissions | `artifacts/worker-permissions.txt`。現在のセッション記録を抽出し確認。設定ファイルだけを証拠にしていない |
| ホストコンパイル | `scripts/android-build.sh --host-only` 成功。最終実行は31秒。`artifacts/android-host-build.log` |
| ビルドスクリプト | `sh -n scripts/android-build.sh`、`shellcheck scripts/android-build.sh` 成功。`artifacts/android-shellcheck.log` |
| ホスト単体テスト | 3件成功、失敗/skipなし。開閉10回と旧画面の操作拒否、復帰/Surface再接続5回と世代、detach失敗時の扱い。記録用のテストruntimeを用いた純粋なKotlin制御の検査であり、JNI/描画実行の代替ではない。`apps/android/app/build/test-results/testDebugUnitTest/` |
| Android lint | 0 errors / 6 warnings。指定targetSdk、arm64限定、旧OSで無視されるBack属性、バックアップ属性、未指定アイコン、プログラム構築UIの警告。`apps/android/app/build/reports/lint-results-debug.txt`。Kotlinコンパイルには旧OS互換のWindowInsets/Back APIの非推奨警告もある |
| JNI署名 | 生成クラスを `javap -s` で確認し、9メソッドがすべて `public static final native`、契約の引数/戻り値であることを確認 |
| Rust cross build | Rust 1.99.0、NDK 28.2.13676358のAPI 26 linker、`--locked --release -p game-runtime --target aarch64-linux-android` 成功。`llvm-readelf -h` でもAArch64/ELF64/DYNを確認 |
| APK生成 | 未完了。基点の共有runtimeはJNI未実装で、スクリプトが `Missing JNI export: Java_lab_rustmobile_NativeBridge_create` を検出しexit 1。`artifacts/android-build.log`。偽stubや不完全なAPKは作成していない |
| Emulator / 実機 / 性能 | 未確認。既存端末は操作していない |

Issue #6は未完了。統合担当は実JNIと描画器を統合し、まずrootで `scripts/android-build.sh`、続けて上記の専用モバイル受入を実施する。JNIのANativeWindow取得保持、同期的な描画無効化、遅延初期化キャンセル、非同期GPU処理と復帰時の時間制御は統合後に確認が必要。共有 `docs/progress.md` は担当範囲外のため変更していない。
