# AI Coding Protocol (ver 6.00)

あなたは私の開発パートナーとして、以下の**「証拠に基づくトップダウン開発」**に従って動作してください。

## 1. 基本コンセプト：動くものが正義

- **まず形にする:** 最速で `src/` 等に「実際に動くコード」を生成せよ。
- **テストが証拠:** 正しく動くことの証明は、言葉ではなく **Playwright（視覚的証拠）** および **Vitest/RTL（論理的証拠）** で行え。
- **進捗の定義:** 監督（ユーザー）が **Playwright のスクリーンショットを見て「意図通りだ」と認めた時**、初めて進捗として記録される。

## 2. 運用ルール

- **@story 準拠:** テストコードは @story の手順と1対1で対応させ、ユーザー体験を保証せよ。
- **JSDocへの集約:** 実装仕様はすべてコード内の JSDoc に集約し、重複するドキュメントは作成しない。
- **名前の絶対遵守:** BOMで決めた命名を絶対とし、実装側で勝手に変更・エイリアス化しないこと。
- **Context Isolation:** 各部品はBOMを通じてのみ通信し、担当範囲外の実装詳細に依存しない。
- **人間の検品:** Playwright は監督へのプレゼンである。画像で意図を証明せよ。

## 3. 標準ディレクトリ構成

    root/
    ├── AGENTS.md            <-- 本規約（AI用OS）
    ├── docs/                <-- 【発注・設計エリア】
    │   ├── bom/             <-- 共通規格：Interface / Zod Schema / State Machine
    │   └── specs/           <-- 発注書：*.spec.ts（主戦場）
    ├── src/                 <-- 【実装・納品エリア】
    ├── tests/               <-- 共通設定・Playwrightビジュアル検品用
    └── context/             <-- コンテクストマッピング用のドキュメントとHTML

## 4. 開発の共通規格：BOM & @story

- **BOM (Bill of Materials):** 実装前に、共有データ、関数の規格、および**状態遷移図（Mermaid）**を `docs/bom/` に確定させよ。
- **発注書 (Spec) メイン:** 開発の主軸は `docs/specs/` 配下の `*.spec.ts` とする。監督が Slot 1 の `@story` を書いた時点で発注確定とする。

## 5. ContextMap.html(Atomic Designの共通イメージ)

大まかな画面レイアウトを示し、分割された領域に必要な機能をリストアップするためのHTMLファイル。これをもとにAIが技術スタックや必要な部品を提案する。対話的にブラッシュアップし、最終的な構成を決定するための「共通のイメージ」を提供する。各要素はコンテクストを構成し、AIはこれをもとに必要な機能を抽出し、BOMやSpecに落とし込む。

```html
<!DOCTYPE html>
<html lang="ja">
  <head>
    <meta charset="UTF-8" />
    <title>Context Map: Vibe to Features</title>
    <!-- グローバルストアの例。実際のアプリでは、これをもとに必要なデータ構造を定義し、BOMに落とし込む。 -->
    <script id="global-store" type="application/json">
      { "currentUser": "John", "theme": "dark", "activeProjectId": null }
    </script>
    <style>
      /* 構造把握のための最低限の装飾 */
      .area {
        border: 1px solid #ccc;
        padding: 20px;
        margin: 10px;
      }
      .features {
        color: #2563eb;
      }
      .features li {
        margin-bottom: 5px;
      }
    </style>
  </head>
  <body>
    <h1>[アプリ名]</h1>
    <p>技術スタック案: React, Lucide, XState</p>

    <nav class="area">
      <h2>Side Navigation</h2>
      <ul class="features">
        <li>[Feature]: プロジェクト一覧を表示し、選択可能にする</li>
        <li>[Feature]: 下部に設定ボタンを配置する</li>
      </ul>
    </nav>

    <main class="area">
      <h2>Main Content</h2>
      <ul class="features">
        <li>[Feature]: 選択中のプロジェクトの詳細をカード形式で表示する</li>
        <li>[Feature]: データの追加・削除ができるフォームを置く</li>
      </ul>
    </main>

    <div class="area" id="sidebar">
      {# サイドバーの状態管理の例 #}
      <script type="application/json">
        { "isCollapsed": false, "hoveredIndex": -1 }
      </script>
      <ul class="features">
        <li>[Feature]: Global Store の activeProjectId を更新する</li>
        <li>[Feature]: isCollapsed が true の時はアイコンのみ表示</li>
      </ul>
    </div>
  </body>
</html>
```

## 6. Specファイルのひな型

AIは、`docs/specs/*.spec.ts` 内に以下の **4スロット** を厳密に構成せよ。

```typescript
/**
 * Slot 1: 発注用ヘッダー (JSDoc Metadata)
 * @context [コンポーネント名/機能名]
 * @bom [docs/bom/ 配下のファイルへのパス]
 * @story
 * 1. [ユーザー操作]
 * 2. [システム/UIの反応]
 * @output [src/ 配下の実装ファイルパス]
 */

/**
 * Slot 2: 外部依存のインポート (Imports)
 */
import { test, expect } from '@playwright/test';
import { render, screen } from '@testing-library/react';
import type { [BOM名] } from '../../bom/[BOMファイル名]';
import { [ComponentName] } from '@output';

/**
 * Slot 3: モック・セットアップ (Test Setup)
 * 実装と検証を切り離すための「独立した基準器」。
 */
const setupMock = () => {
  const mockData: [BOM名] = {
    // BOMに基づいたダミーデータ定義
  };
  return mockData;
};

/**
 * Slot 4: 挙動の検証コード (Story Verification)
 */

// --- 1. AIの内省 (Logic Verification) ---
// AIが自己修正ループを回し、ロジックを固めるための高速ループ。
// test('logic: should handle state transition', () => { ... });

// --- 2. 監督へのプレゼン (Visual Story) ---
test.describe('[Context] Visual Story', () => {
  test('should satisfy the story steps with evidence', async ({ page }) => {
    // @story の手順を実演し、スクリーンショットを撮影。
    // 修正時は `result_before.png` と `result_after.png` を出力し比較可能にせよ。
    await page.screenshot({ path: `evidence/[Context]_result.png` });
  });
});
```

## 7. 実行ワークフロー（フェーズ分離型）

本プロトコルは**設計フェーズ**と**実装フェーズ**に分離する。
Specが両フェーズの**境界面**となる。

```
ContextMap.html → BOM → Spec  ←【境界面】→  実装 → 証拠
```

---

### 【設計フェーズ】設計モデルを使用

設計モデルはContextMap.htmlを読み、往復を最小化してBOM・Specを生成する。
抽象的な指示から具体的な設計を導出する能力が必要。

1. **概要説明:** ContextMap.htmlを用いてアプリの概要と画面構成を説明する。
2. **構成提案:** 設計モデルはContextMap.htmlから技術スタックを提案する。
3. **構成決定:** 人間はAIの提案をブラッシュアップし、必要な部品を決定する。
4. **BOM作成:** 設計モデルは部品をBOM（Zod Schema/Type）として `docs/bom/` に生成する。
5. **状態定義:** 設計モデルは状態遷移をMermaid等で可視化し、BOMの一部として保存する。
6. **Spec作成:** 設計モデルはSpecファイルの Slot 1 に @story を記載する。
7. **ひな型完成:** 設計モデルはSpec ファイルの全Slotを埋め、テストの「器」を完成させる。
8. **フロー承認:** 人間はSpecを確認し、実装の開始を承認する。← **設計フェーズの完了条件**

---

### 【実装フェーズ】実装モデルを使用

実装モデルはSpecを受け取り、忠実にコードを生成する。
Specが十分に具体的であれば、追加のプロンプトエンジニアリングは不要。

9. **実装実行:** 実装モデルはSpecの @story を元に `src/` にコードを実装する。
10. **内省ループ:** 実装モデルは自らテスト（RTL等）を実行し、ロジックの不備を自己修正する。
11. **ビジュアル出力:** 実装モデルはPlaywrightでスクリーンショットを撮影する。
12. **人間検品:** 人間が画面を確認。意図と異なる場合はAIが修正し、`evidence/` ディレクトリに `result_before.png`（修正前）および `result_after.png`（修正後）を保存し再提出する。
13. **ロジック深掘り:** 見た目では分からないエッジケースのテストを追加し、堅牢にする。
14. **パス確認:** 人間が全テストのパスを確認し、承認する。← **実装フェーズの完了条件**

---

### 【次コンテキストへ】

15. **次コンテキスト:** storyが満たされたら、設計フェーズの手順3（構成決定）に戻り、次のコンテキストを開始する。
