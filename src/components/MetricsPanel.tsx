import { useEffect, useState } from 'react'

import { fetchFinancialMetrics, type FinancialMetricRecord } from '@/lib/financialMetrics'
import type { Selection } from '@/store/useKabutoStore'

export interface MetricsPanelProps {
    /** 選択中の銘柄。nullのときは未選択のプレースホルダーを表示する */
    selection: Selection | null
    onFetchMetrics?: (companyCode: string) => Promise<FinancialMetricRecord[]>
}

const PLACEHOLDER_CLASS =
    'flex items-center justify-center h-full text-(--kabuto-fg-dim) text-[12px] font-mono'

/**
 * 選択中銘柄の財務指標を、指標×決算年度の表で表示する（2026/08/18/002.md Step 7）。
 * 5期比較サマリー由来のデータなので、指標を行・決算年度を列にしたピボット表にする。
 */
export function MetricsPanel({ selection, onFetchMetrics = fetchFinancialMetrics }: MetricsPanelProps) {
    const [metrics, setMetrics] = useState<FinancialMetricRecord[]>([])
    const [isLoading, setIsLoading] = useState(false)

    useEffect(() => {
        if (!selection) {
            setMetrics([])
            return
        }
        let cancelled = false
        setIsLoading(true)
        onFetchMetrics(selection.code)
            .then((rows) => {
                if (!cancelled) setMetrics(rows)
            })
            .finally(() => {
                if (!cancelled) setIsLoading(false)
            })
        return () => {
            cancelled = true
        }
    }, [selection, onFetchMetrics])

    if (!selection) {
        return (
            <div data-testid="metrics-panel-placeholder" className={PLACEHOLDER_CLASS}>
                銘柄を選択してください
            </div>
        )
    }

    if (isLoading) {
        return (
            <div data-testid="metrics-panel-loading" className={PLACEHOLDER_CLASS}>
                読み込み中...
            </div>
        )
    }

    if (metrics.length === 0) {
        return (
            <div data-testid="metrics-panel-empty" className={PLACEHOLDER_CLASS}>
                {selection.name}（{selection.code}）の財務データはありません
            </div>
        )
    }

    const years = [...new Set(metrics.map((m) => m.fiscal_year))].sort((a, b) => a - b)
    const metricNames = [...new Set(metrics.map((m) => m.metric))]
    const byMetricYear = new Map<string, Map<number, FinancialMetricRecord>>()
    for (const m of metrics) {
        if (!byMetricYear.has(m.metric)) byMetricYear.set(m.metric, new Map())
        byMetricYear.get(m.metric)!.set(m.fiscal_year, m)
    }

    return (
        <div data-testid="metrics-panel-table" className="h-full overflow-auto">
            <div className="px-3 py-2 font-mono text-[9px] font-bold tracking-[0.15em] uppercase text-(--kabuto-fg-dim) bg-(--kabuto-panel) border-b border-(--kabuto-border) sticky top-0">
                {selection.name}（{selection.code}）財務指標
            </div>
            <table className="w-full text-[11px] font-mono border-collapse">
                <thead>
                    <tr className="border-b border-(--kabuto-border)">
                        <th className="text-left px-3 py-1.5 text-(--kabuto-fg-dim) font-semibold">指標</th>
                        {years.map((y) => (
                            <th key={y} className="text-right px-3 py-1.5 text-(--kabuto-fg-dim) font-semibold">
                                {y}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {metricNames.map((name) => (
                        <tr
                            key={name}
                            data-testid={`metrics-row-${name}`}
                            className="border-b border-(--kabuto-border)"
                        >
                            <td className="px-3 py-1.5 text-(--kabuto-fg)">{name}</td>
                            {years.map((y) => {
                                const rec = byMetricYear.get(name)?.get(y)
                                return (
                                    <td key={y} className="text-right px-3 py-1.5 text-(--kabuto-fg)">
                                        {rec ? `${rec.value.toLocaleString()} ${rec.unit}` : '—'}
                                    </td>
                                )
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    )
}
