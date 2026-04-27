---
name: context-mapping
description: アプリのUI構成を示すHTMLファイルを用いて、AIにアプリの概要と画面構成を説明するためのガイドライン。
　- **目的:** 画面構成のイメージを示す「ContextMap.html」（アプリのUI構成を示すHTMLファイル）等を用いて、AIにアプリの概要と画面構成を説明する。
---

# context-mapping

ユーザーとのやり取りからアプリの画面構成と必要な機能を抽出しコンテクスト・マップを作成するための手順を示すガイドライン。

## Prerequisites

- アプリケーションの概要、または主要な機能のラフな記述。

## Logic / Procedure

### 1. Context Identification

ユーザーの記述から、独立してテスト・開発可能な「エリア（コンテクスト）」を特定する。

- セマンティックなHTMLタグ（`nav`, `main`, `aside`, `footer`, `header`）で構造化する
- 各エリアに `id="ctx-[name]"` を付与する
- 各エリアの責務を HTMLコメントで明記する

```html
<!-- ============================================================
     CTX-N: [NAME]
     責務: [このエリアが担う役割]
     [その他、実装上重要な制約や注意点があれば記述する]
============================================================ -->
```

### 2. State Hierarchy Planning

**Global Store** (`script#global-store`):

- 複数のコンテキストをまたいで共有すべき状態のみを `<head>` 内に配置する
- 何を共有すべきかはアプリの性質による（例: セッション、選択状態、ルーティング）
- 完全にローカルで動作するアプリや、コンテキスト間で共有すべき状態がない場合は省略してよい
  **Local State** (`script.local-state`):
- 各コンテキスト内に閉じる状態を、該当エリア内の `script` タグに配置する
  **拡張フィールド（必要な場合のみ追加する）:**

| フィールド        | 用途                             | 追加条件                       |
| ----------------- | -------------------------------- | ------------------------------ |
| `uiDataFlows`     | コンテキスト間のデータフロー定義 | 複数コンテキストが連動する場合 |
| `agent`           | LLMのモデル・設定                | LLMを使う場合                  |
| `actionRegistry`  | LLMが呼び出せる関数の定義        | LLMにツールを使わせる場合      |
| `operationPolicy` | 操作の危険度分類                 | 外部DBへの書き込みを伴う場合   |

### 3. Data Flow の定義（必要な場合）

コンテキスト間にデータの流れがある場合、`uiDataFlows` として明示する。
**UIイベント起点のフローのみ**を記述する。DBアクセスは各コンテキストの責務コメントに記述する。

```json
"uiDataFlows": [
  {
    "from":    "[送信元コンテキストID]",
    "to":      "[送信先コンテキストID]",
    "data":    "[送信するデータ]",
    "trigger": "[発火条件]"
  }
]
```

### 4. Feature Extraction

各コンテキストの役割を以下の形式で箇条書きに変換する。

- `[Feature]` — ユーザーに見える機能
- `[Logic]` — システム内部の振る舞い

```html
<ul class="features">
  <li>
    <b>[Feature] [機能名]</b>
    <p>具体的な説明。</p>
  </li>
  <li>
    <b>[Logic] [ロジック名]</b>
    <p>具体的な説明。</p>
  </li>
</ul>
```

### 5. HTML Construction

- セマンティックなHTMLタグ（`nav`, `main`, `aside`等）で構造化する
- 主要なUIのモックアップ（ダミーデータ・ボタン等）を埋め込み、実装イメージを伝える
- デザイン上のこだわりはHTMLコメントとして記述する
- CSSはContextMapの視認性のために最低限定義する（実装への制約ではない）

---

## Outputs

`ContextMap.html`:

- ブラウザで表示可能なプロトタイプ
- 各コンテキストの責務コメント
- `[Feature]` / `[Logic]` による機能リスト（BOM・Specの種）
- 必要に応じた `script#global-store` による状態定義
- AIエージェントがパース可能な `script` タグによる状態定義を内包

---

## Example Output

```html
<!DOCTYPE html>
<html lang="ja">
  <head>
    <meta charset="UTF-8" />
    <title>Mockup: [Context Name]</title>
    <!-- アプリ全体で共有すべき状態はここに定義する。AIはこれをもとにGlobal
    Storeの構造を提案する。 -->
    <script id="global-store" type="application/json">
      {
        "session": { "user": "Guest", "isLoggedIn": false },
        "theme": "light",
        "routing": { "currentPath": "/dashboard" }
      }
    </script>

    <style>
      :root {
        --primary: #2563eb;
        --gap: 1rem;
      }
      .context-area {
        border: 2px solid #eee;
        padding: var(--gap);
        margin: var(--gap);
        border-radius: 8px;
      }
      .features {
        color: var(--primary);
        font-family: monospace;
        background: #f8fafc;
        padding: 1rem;
        list-style: none;
      }
    </style>
  </head>

  <body>
    <!-- ============================================================
         CTX-1: SIDEBAR
         責務: ナビゲーション・ルーティング更新
    ============================================================ -->
    <nav id="ctx-sidebar" class="context-area">
      <h2>Navigation</h2>
      <!-- コンテクスト内で閉じる状態はここに定義する。AIはこれをもとにLocal
      Stateの構造を提案する。 -->
      <script class="local-state" type="application/json">
        { "items": ["Home", "Settings"], "activeIdx": 0 }
      </script>

      <ul class="features">
        <li>[Feature] itemsをループで描画し、activeIdxを強調表示する</li>
        <li>[Logic] クリック時に global-store の routing を更新する</li>
      </ul>
    </nav>

    <!-- ============================================================
         CTX-2: MAIN
         責務: 選択中のルートに応じたコンテンツ表示
    ============================================================ -->
    <main id="ctx-main" class="context-area">
      <h2>Main Display</h2>
      <ul class="features">
        <li>[Feature] 選択されたPathに応じたコンテンツを動的に表示する</li>
        <li>
          [Logic] ログイン状態(global-store)に応じて表示内容を認可制御する
        </li>
      </ul>
    </main>

    <!--
    追加のコンテクストがあれば同様に定義する。必要に応じて、コンテクスト間のデータフローを
    script#global-store 内の uiDataFlows で定義する。
    例:
    "uiDataFlows": [
        {
            "from": "ctx-sidebar",
            "to": "ctx-main",
            "data": "activeIdx",
            "trigger": "クリックイベント"
        }
    ]
    -->
  </body>
</html>
```

---

## Notes

**アプリの性質に応じて取捨選択する:**

- シンプルなローカルアプリ → global-storeは最小限またはなし
- LLMを使うアプリ → `agent`・`actionRegistry`等をglobal-storeに追加
- 外部DBへの書き込みを伴うアプリ → `operationPolicy`をglobal-storeに追加
- コンテキスト間の連動が多いアプリ → `uiDataFlows`を追加
  **曖昧でよいもの:**
- 型定義の詳細（LLMが推論する）
- バックエンドの実装詳細（Specで定義する）
- 将来追加される機能（育ってから追加する）
