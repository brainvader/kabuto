use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    #[serde(rename = "docID")]
    pub doc_id: String,
    pub edinet_code: Option<String>,
    pub sec_code: Option<String>,
    pub filer_name: Option<String>,
    pub ordinance_code: Option<String>,
    pub form_code: Option<String>,
    pub doc_description: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub submit_date_time: Option<String>,
    pub seq_number: Option<u64>,
    #[serde(rename = "JCN")]
    pub jcn: Option<String>,
    pub fund_code: Option<String>,
    pub doc_type_code: Option<String>,
    pub issuer_edinet_code: Option<String>,
    pub subject_edinet_code: Option<String>,
    pub subsidiary_edinet_code: Option<String>,
    pub current_report_reason: Option<String>,
    #[serde(rename = "parentDocID")]
    pub parent_doc_id: Option<String>,
    pub ope_date_time: Option<String>,
    pub withdrawal_status: Option<String>,
    pub doc_info_edit_status: Option<String>,
    pub disclosure_status: Option<String>,
    pub xbrl_flag: Option<String>,
    pub pdf_flag: Option<String>,
    pub attach_doc_flag: Option<String>,
    pub english_doc_flag: Option<String>,
    pub csv_flag: Option<String>,
    pub legal_status: Option<String>,
}

impl Document {
    pub fn is_annual_report(&self) -> bool {
        self.ordinance_code.as_deref() == Some("010") && self.form_code.as_deref() == Some("030000")
    }
}

pub struct Filter {
    pub edinet_codes: Vec<String>,
    pub sec_codes: Vec<String>,
    pub names: Vec<String>,
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.edinet_codes.is_empty() && self.sec_codes.is_empty() && self.names.is_empty()
    }

    pub fn matches(&self, doc: &Document) -> bool {
        if self.is_empty() {
            return true;
        }
        self.edinet_codes
            .iter()
            .any(|c| doc.edinet_code.as_deref() == Some(c.as_str()))
            || self
                .sec_codes
                .iter()
                .any(|c| doc.sec_code.as_deref() == Some(c.as_str()))
            || self.names.iter().any(|n| {
                doc.filer_name
                    .as_deref()
                    .map(|f| f.contains(n.as_str()))
                    .unwrap_or(false)
            })
    }
}
