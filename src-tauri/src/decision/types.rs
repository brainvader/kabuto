//! decision / decision_session / trigger の型定義（2026/08/15/002・003・004.md）。
//! フィールド構成は schema.surql と対応させている。

use serde::{Deserialize, Serialize};

/// 1つの仮説検証の結果。based_on_doc_ids で financial_metric/disclosure_text の
/// 元データまで遡れる。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub hypothesis: String,
    pub method: String,
    pub correlation: Option<f64>,
    #[serde(default)]
    pub confirmed: bool,
    pub lag_days: Option<i32>,
    pub based_on_doc_ids: Vec<String>,
}

/// 選択肢の集合と結論。
/// selection_mode: "exclusive"（排他的候補から1つ採用） | "composite"（複数要因を統合）
/// action_taken:   "bought" | "sold" | "watched" | "none"
///   Palantir流の承認ゲートではなく、事後に振り返るための事実の記録（08/15/004.md）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSession {
    pub question: String,
    /// company レコードへの参照（例: "company:1605"）
    pub company: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_selection_mode")]
    pub selection_mode: String,
    pub conclusion: Option<String>,
    #[serde(default = "default_action_taken")]
    pub action_taken: String,
}

fn default_status() -> String {
    "exploring".to_string()
}

fn default_selection_mode() -> String {
    "exclusive".to_string()
}

fn default_action_taken() -> String {
    "none".to_string()
}

/// list_decision_sessions の1行分。DecisionSession に record id を加えたもの
/// （一覧からセッションを選ぶUIにはidが必須だが、CONTENT書き込みに使う
/// DecisionSession自体にはSCHEMAFULL上idフィールドが無いため分けている）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSessionSummary {
    pub id: String,
    pub question: String,
    pub company: Option<String>,
    pub status: String,
    pub selection_mode: String,
    pub conclusion: Option<String>,
    pub action_taken: String,
}

/// decision_session の CONSIDERED エッジ1本分。target は "decision:xxx" | "decision_session:xxx"
/// 形式のレコード参照文字列（呼び出し側が ':' で table/id に分解する）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsideredEdge {
    pub target: String,
    pub selected: bool,
    pub rejection_reason: Option<String>,
}

/// 「なぜこの仮説を思いついたか」という思考の切っ掛け（08/15/003.md）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    #[serde(rename = "type")]
    pub kind: String, // human_note | news | disclosure_text_match | precedent_recall
    pub description: String,
    pub source_ref: Option<String>,
}
