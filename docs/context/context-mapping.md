---
name: context-mapping
description:
  アプリのUI構成を示すHTMLファイルを用いて、AIにアプリの概要と画面構成を説明するためのガイドライン。
  - **目的:** 画面構成のイメージを示す「ContextMap.html」をデザインカンプとして作成し、Vitestテストファイルと対応させる。
---

# context-mapping

ユーザーとのやり取りからアプリの画面構成と必要な機能を抽出し、ContextMap.htmlを作成するための手順を示すガイドライン。

## Prerequisites

- アプリケーションの概要、または主要な機能のラフな記述。

---

## 基本方針

- **ContextMap.html はデザインカンプ** — 表示要素でUIを表現し、コメントでコンテキストを定義する
- **Vitest テストファイルがコンテキストのSSOT** — `import` がスコープを、`describe`/`it` が振る舞いを定義する
- **ContextMap はテストの目次** — 各コンポーネントのコメントに `describe`/`it` を列挙し、`[✓]`/`[ ]` でチェックする
- **CTX番号による管理は不要** — テストファイルのパスで対応を示す

---

## Logic / Procedure

### 1. HTML構造の作成

セマンティックなHTMLタグ（`nav`, `main`, `aside`, `footer`, `header`）で画面構造を作る。
各要素の `id` はコンポーネント名をケバブケースにしたものを使う。

```html
<header id="topbar">...</header>
<main id="project-grid">...</main>
<dialog id="settings-dialog">...</dialog>
```

---

### 2. State Hierarchy Planning

#### Global Store（`script#global-store`）

複数のコンテキストをまたいで共有すべき状態のみを `<head>` 内に配置する。

```html
<!-- ============================================================
     GLOBAL STORE
     複数のコンテキストをまたいで共有すべき状態のみを定義する。
     routing は router ライブラリに委譲するため持たない。
============================================================ -->
<script id="global-store" type="application/json">
  { "activeProjectId": null }
</script>
```

#### Local State（`script.local-state`）

各コンポーネント内に閉じる状態は `<head>` 内に定義する。`data-context` で対応するidと紐付ける。

```html
<!-- [component-id] のローカル状態 -->
<script data-context="project-grid" class="local-state" type="application/json">
  {
    "isNewProjectDialogOpen": false,
    "form": { "name": "", "description": "", "rootPath": "" },
    "errors": { "name": null, "rootPath": null }
  }
</script>
```

---

### 3. Stack の定義

技術スタックは `<head>` 内にコメントとして記述する。

```html
<!-- ============================================================
     STACK
     runtime:  [例: Tauri 2]
     frontend: [例: React 18 + TypeScript]
     router:   [例: TanStack Router（routing の SSOT）]
     state:    [例: Zustand]
     db:       [例: SurrealDB embedded]
     testing:  [例: Vitest + RTL + Storybook + Playwright]
     package:  [例: pnpm]
============================================================ -->
```

---

### 4. describe/it コメントの記述

各コンポーネントに対応するHTML要素のそばに、テストファイルのパスと `describe`/`it` を列挙する。
テストがグリーンになったら `[✓]`、未実装は `[ ]`。

```html
<!-- src/components/FileTree.tsx
  describe: ファイルツリーを閲覧・選択する
    [✓] rootPath 配下のエントリがツリー表示される
    [✓] ディレクトリをクリックすると子エントリが遅延ロードされ展開される
    [ ] 新しい機能の説明
-->
<nav id="file-tree">
  <!-- モックアップ -->
</nav>
```

**ルール:**

- `describe` はユーザーの行動を軸にした日本語で記述する（例: 「ファイルツリーを閲覧・選択する」）
- `it` はVitestのテストファイルの文言と1対1で対応させる
- 未実装のコンポーネントはファイルパスのみ記述し、`[ ]` で列挙する

---

### 5. Overlay（Dialog / Modal）の配置

`position: fixed` が親要素の `overflow` に封じられないよう、**必ず `</body>` 直前**に配置する。

```html
<!-- src/components/SettingsDialog.tsx
  describe: 設定を確認・閉じる
    [✓] open=true のとき「Settings」タイトルと「閉じる」ボタンが表示される
    [✓] open=false のときコンテンツが表示されない
    [✓] 「閉じる」ボタンをクリックすると onOpenChange(false) が呼ばれる
-->
<div class="dialog-overlay">
  <dialog
    style="all:unset; display:flex; flex-direction:column; box-sizing:border-box;"
  >
    <!-- モックアップ -->
  </dialog>
</div>
```

---

## Outputs

`ContextMap.html`:

- ブラウザで表示可能なデザインカンプ
- 各コンポーネントの `describe`/`it` コメント（テストの目次）
- `[✓]`/`[ ]` による実装状況のチェックリスト
- `<head>` 内の `script#global-store` および `script.local-state` による状態定義
- STACKコメントによる技術スタック定義

---

## Example Output

```html
<!DOCTYPE html>
<html lang="ja">
  <head>
    <meta charset="UTF-8" />
    <title>ContextMap: [アプリ名]</title>

    <!-- ============================================================
         STACK
         runtime:  Tauri 2
         frontend: React 18 + TypeScript
         router:   TanStack Router（routing の SSOT）
         state:    Zustand
         testing:  Vitest + RTL + Storybook + Playwright
         package:  pnpm
    ============================================================ -->

    <!-- ============================================================
         GLOBAL STORE
         複数のコンテキストをまたいで共有すべき状態のみを定義する。
    ============================================================ -->
    <script id="global-store" type="application/json">
      { "projects": [] }
    </script>

    <!-- project-grid のローカル状態 -->
    <script
      data-context="project-grid"
      class="local-state"
      type="application/json"
    >
      {
        "isNewProjectDialogOpen": false,
        "form": { "name": "", "description": "", "rootPath": "" },
        "errors": { "name": null, "rootPath": null }
      }
    </script>

    <style>
      /* デザインカンプ用スタイル */
    </style>
  </head>
  <body>
    <!-- src/components/Topbar.tsx
      describe: 設定を開く
        [✓] ロゴ・タイトル・バージョンが表示される
        [✓] Settings ボタンが表示される
        [✓] Settings ボタンをクリックすると onSettingsClick が呼ばれる
    -->
    <header id="topbar">
      <!-- モックアップ -->
    </header>

    <!-- src/components/ProjectGrid.tsx
      describe: プロジェクトを一覧・作成する
        [✓] projects が空のとき「＋ new project」ボタンのみ表示される
        [✓] projects があるときカードが表示される
        [✓] 「＋ new project」をクリックするとダイアログが開く
        [ ] 新しい機能の説明
    -->
    <main id="project-grid">
      <!-- モックアップ -->
    </main>

    <!-- src/components/SettingsDialog.tsx
      describe: 設定を確認・閉じる
        [✓] open=true のとき「Settings」タイトルと「閉じる」ボタンが表示される
        [✓] 「閉じる」ボタンをクリックすると onOpenChange(false) が呼ばれる
    -->
    <div class="dialog-overlay">
      <dialog
        style="all:unset; display:flex; flex-direction:column; box-sizing:border-box;"
      >
        <!-- モックアップ -->
      </dialog>
    </div>
  </body>
</html>
```

---

## Notes

**曖昧でよいもの:**

- 型定義の詳細（Vitestのテストを書く過程で自然に定まる）
- 将来追加される機能（`[ ]` で列挙しておき、実装時に `[✓]` にする）

**Vitestとの関係:**

- ContextMapは「テストの目次」であり、Vitestテストファイルがコンテキストのオリジナル
- ContextMapを先に作り、それをもとにVitestテストを生成する
- テストがグリーンになったら ContextMap の `[ ]` を `[✓]` に更新する
- 仕様変更時はVitestテストを先に変更し、ContextMapを追従させる
