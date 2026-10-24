# SEED Race | アセット・権利・制作フロー
更新: 2026-10-24。デザイン探索のルールであり、**本番公開の許諾ではない**。

## ビジュアル正本
ユーザーから共有された「SEED + 9 companions / idle / dead / win / fixed feet and body」の画像を基準にする。既存名: seed, noppo, maru, nagi, koro, saya, goma, futa, sumi, shiro。
- 大きく変えない: 頭頂の形、輪郭、足、色、顔の簡素さ、濃い線、個体ごとの縦横比、idle/dead/winの表情。
- SEED1体で**2D billboardと3Dトゥーン**を同じコース・同じカメラで動かして比較。何を採用したか明示し、無採用の仮画像を正本として使わない。
- 走る/滑る/跳ねる/衝突する演出は身体の変形・傾き・表情で付ける。物理コライダーは同一形状/質量/摩擦を初期値にし、キャラ見た目に公平性を左右させない。
- 残る9体は技術と美術の1体受入後に製作する。色違いとキャラ固有形状を混同しない。

## 公開と権利の境界
このrepoはPublic。添付原画自体/切抜き/自動トレース/3D派生の公開権利は未確認。**画像をリポジトリへ取り込まない**。公開検証には自作の権利問題のない合成ダミーを使い、公式SEED素材が必要な試行は許可されたPrivate環境に置く。Avvy/AnotherBallの社内コード・ギフト・FXは私有repoへ無断流入させない。ライセンス・出典・美術採用・技術合格・実アプリ合格を別々に記録する。

## 製作の一巡
1. Brief/リファレンス(出典と公開可否付き)を登録する。
2. `creative-lab`で試作を比較 (Blender実レンダー/Three.js固定撮影/FX/compare)。固定commitと外部環境バージョンを記録。
3. Blenderの編集可能な原本（.blendなど）と、GLB/texture/アニメーション/簡略Collider/効果音を別管理する。
4. Publicに置けるContent Packは `manifest`にformat_version, source_revision, creator, rights_status, file_sha256, canvas/pivot or units, LOD, texture_budget, animation clips, collider_revision, review_status を記録する。ライセンス未確定のPackは取り込まない。
5. Runtimeで一回表示し固定時刻画像/短い動画とメモリ・負荷を確認。比較結果は`atelier-asset-browser`へ画像/動画で渡せる（直接GLB表示できると仮定しない）。
6. 人による美術採用・遊びの視認性・モバイル実再生の判定後に、ゲームの正本へ昇格する。

## 初期ステージの方針
Marbles等の他社コースの複製は禁止。からくり玩具風は検討候補の一つで、他の2案と同じ固定条件で比較してG04/G05で採否を決める。独自のコース1つから始め、形状・物理Collider・演出・撮影用カメラを再利用しやすい部品単位にする。

## エフェクトと音
FXは見た目が美しくても原画/名前/順位を覆わないこと。alpha/色空間/発光/ループ/音量/ライセンスを個別にQAする。音制作に共通の完成基盤があるとは主張せず、まずゲーム側で測る。
