import { invoke } from '@tauri-apps/api/core'

/** 財務指標1件（src-tauri/src/financial_data/types.rs の FinancialMetricRecord と対応） */
export interface FinancialMetricRecord {
    doc_id: string
    company_code: string | null
    metric: string
    xbrl_tag: string
    value: number
    unit: string
    fiscal_year: number
    consolidated: boolean
}

/** 指定した証券コードの財務指標を、決算年度の昇順で取得する。 */
export function fetchFinancialMetrics(companyCode: string): Promise<FinancialMetricRecord[]> {
    return invoke('list_financial_metrics_cmd', { companyCode })
}
