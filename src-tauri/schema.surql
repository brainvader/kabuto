-- KABUTO SurrealDBスキーマ
-- src-tauri/src/lib.rs（Tauriアプリ）と scripts/fetch_xbrl（ingestコマンド）の
-- 両方から include_str! で読み込む単一のスキーマ定義。
-- DEFINE ... OVERWRITE により、再適用しても壊れない（冪等）。

-- company ノード（2026/08/18/002.md）
-- 証券コードを主キーとする（type::record('company', code)）。
-- 様々なデータ（financial_metric/disclosure_text等）が統合される中心ノード。
DEFINE TABLE OVERWRITE company SCHEMAFULL;
DEFINE FIELD OVERWRITE code        ON TABLE company TYPE string;
DEFINE FIELD OVERWRITE name        ON TABLE company TYPE string;
DEFINE FIELD OVERWRITE sector      ON TABLE company TYPE option<string>;  -- 33業種区分
DEFINE FIELD OVERWRITE market      ON TABLE company TYPE option<string>;  -- 市場区分（プライム/スタンダード/グロース）
DEFINE FIELD OVERWRITE edinet_code ON TABLE company TYPE option<string>;  -- XBRL取得元への遡及用
DEFINE FIELD OVERWRITE updated_at  ON TABLE company TYPE datetime DEFAULT time::now();

-- macro ノード（MacroIndicator）
-- 将来: WTI / USDJPY / NIKKEI / FED_RATE など
DEFINE TABLE OVERWRITE macro SCHEMAFULL;
DEFINE FIELD OVERWRITE name     ON TABLE macro TYPE string;
DEFINE FIELD OVERWRITE category ON TABLE macro TYPE string; -- COMMODITY | CURRENCY | INDEX | INTEREST_RATE
DEFINE FIELD OVERWRITE source   ON TABLE macro TYPE string;

-- financial_metric ノード（2026/08/14/005.md）
-- レコードIDは ⟨doc_id⟩_⟨metric⟩_⟨fiscal_year⟩ の複合キー。
-- 1つのXBRL提出書類（doc_id）から複数レコードが生成される。
DEFINE TABLE OVERWRITE financial_metric SCHEMAFULL;
DEFINE FIELD OVERWRITE doc_id       ON TABLE financial_metric TYPE string;
DEFINE FIELD OVERWRITE company      ON TABLE financial_metric TYPE option<record<company>>;  -- companyへのレコードリンク（2026/08/18/002.md）
DEFINE FIELD OVERWRITE metric       ON TABLE financial_metric TYPE string;  -- 正規化した指標名
DEFINE FIELD OVERWRITE xbrl_tag     ON TABLE financial_metric TYPE string;  -- 元のタクソノミ要素名
DEFINE FIELD OVERWRITE value        ON TABLE financial_metric TYPE float;
DEFINE FIELD OVERWRITE unit         ON TABLE financial_metric TYPE string;
DEFINE FIELD OVERWRITE fiscal_year  ON TABLE financial_metric TYPE int;
DEFINE FIELD OVERWRITE consolidated ON TABLE financial_metric TYPE bool;
-- companyのみインデックス化する（ダッシュボードの銘柄別検索用）。doc_idは主キー
-- 直接指定でアクセスするため不要（2026/08/17/003.md：検索ベース更新のO(n²)バグの教訓）。
DEFINE INDEX OVERWRITE financial_metric_company ON TABLE financial_metric COLUMNS company;

-- disclosure_text ノード（2026/08/14/005.md）
-- レコードIDは ⟨doc_id⟩_⟨section⟩ の複合キー。
-- embeddingはOpenAI text-embedding-3-small。対象は自然言語セクションのみ。
DEFINE TABLE OVERWRITE disclosure_text SCHEMAFULL;
DEFINE FIELD OVERWRITE doc_id      ON TABLE disclosure_text TYPE string;
DEFINE FIELD OVERWRITE company     ON TABLE disclosure_text TYPE option<record<company>>;  -- companyへのレコードリンク（2026/08/18/002.md）
DEFINE FIELD OVERWRITE section     ON TABLE disclosure_text TYPE string;
DEFINE FIELD OVERWRITE fiscal_year ON TABLE disclosure_text TYPE int;
DEFINE FIELD OVERWRITE text        ON TABLE disclosure_text TYPE string;
DEFINE FIELD OVERWRITE embedding   ON TABLE disclosure_text TYPE array<float>;
-- companyのみインデックス化する。doc_idは主キー直接指定でアクセスするため不要
-- （2026/08/17/003.md：検索ベース更新のO(n²)バグの教訓）。
DEFINE INDEX OVERWRITE disclosure_text_company ON TABLE disclosure_text COLUMNS company;

-- trigger ノード（2026/08/15/003.md）
-- 「なぜこの仮説を思いついたか」という思考の切っ掛けを記録する。
DEFINE TABLE OVERWRITE trigger SCHEMAFULL;
DEFINE FIELD OVERWRITE type        ON TABLE trigger TYPE string; -- human_note | news | disclosure_text_match | precedent_recall
DEFINE FIELD OVERWRITE description ON TABLE trigger TYPE string;
DEFINE FIELD OVERWRITE source_ref  ON TABLE trigger TYPE option<string>;
DEFINE FIELD OVERWRITE created_at  ON TABLE trigger TYPE datetime DEFAULT time::now();

-- decision ノード（2026/08/15/002.md）
-- 1つの仮説検証の結果。based_on_doc_ids で financial_metric/disclosure_text の元データまで遡れる。
DEFINE TABLE OVERWRITE decision SCHEMAFULL;
DEFINE FIELD OVERWRITE hypothesis       ON TABLE decision TYPE string;
DEFINE FIELD OVERWRITE method           ON TABLE decision TYPE string;
DEFINE FIELD OVERWRITE correlation      ON TABLE decision TYPE option<float>;
DEFINE FIELD OVERWRITE confirmed        ON TABLE decision TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE lag_days         ON TABLE decision TYPE option<int>;
DEFINE FIELD OVERWRITE based_on_doc_ids ON TABLE decision TYPE array<string>;
DEFINE FIELD OVERWRITE computed_at      ON TABLE decision TYPE datetime DEFAULT time::now();

-- decision_session ノード（2026/08/15/002・003・004.md）
-- 選択肢の集合と結論。旧 company->AFFECTED_BY->macro はこの仕組みに統合し廃止した。
DEFINE TABLE OVERWRITE decision_session SCHEMAFULL;
DEFINE FIELD OVERWRITE question       ON TABLE decision_session TYPE string;
DEFINE FIELD OVERWRITE company        ON TABLE decision_session TYPE option<record<company>>;
DEFINE FIELD OVERWRITE status         ON TABLE decision_session TYPE string DEFAULT 'exploring';  -- exploring | resolved
DEFINE FIELD OVERWRITE selection_mode ON TABLE decision_session TYPE string DEFAULT 'exclusive';  -- exclusive | composite
DEFINE FIELD OVERWRITE conclusion     ON TABLE decision_session TYPE option<string>;
DEFINE FIELD OVERWRITE action_taken   ON TABLE decision_session TYPE string DEFAULT 'none';  -- bought | sold | watched | none
DEFINE FIELD OVERWRITE created_at     ON TABLE decision_session TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE resolved_at    ON TABLE decision_session TYPE option<datetime>;

-- RELATE エッジ
-- TYPE RELATION を付けないと RELATE 時に TableCheck エラーになる（NORMAL テーブル扱いのため）。
-- has_metric / has_disclosure は使わず、financial_metric/disclosure_text側の
-- company（record<company>）フィールドで直接参照する方式に統一した（2026/08/18/002.md）。

-- prompted: trigger が decision_session を誘発したことを表す
DEFINE TABLE OVERWRITE prompted TYPE RELATION SCHEMAFULL;

-- considered: decision_session が検討した候補。decision にも decision_session にも
-- RELATE できる（ネストした判断を許容するため、IN/OUTの型は固定しない）。
DEFINE TABLE OVERWRITE considered TYPE RELATION SCHEMAFULL;
DEFINE FIELD OVERWRITE selected         ON TABLE considered TYPE bool;
DEFINE FIELD OVERWRITE rejection_reason ON TABLE considered TYPE option<string>;
