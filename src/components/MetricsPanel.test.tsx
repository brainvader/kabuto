import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'

import { MetricsPanel } from '@/components/MetricsPanel'
import type { FinancialMetricRecord } from '@/lib/financialMetrics'

const mockMetrics: FinancialMetricRecord[] = [
    {
        doc_id: 'S100YK0W',
        company_code: '1375',
        metric: 'Revenue',
        xbrl_tag: 'RevenueSummaryOfBusinessResults',
        value: 47081000000,
        unit: 'JPY',
        fiscal_year: 2022,
        consolidated: true,
    },
    {
        doc_id: 'S100YK0W',
        company_code: '1375',
        metric: 'Revenue',
        xbrl_tag: 'RevenueSummaryOfBusinessResults',
        value: 53449000000,
        unit: 'JPY',
        fiscal_year: 2026,
        consolidated: true,
    },
    {
        doc_id: 'S100YK0W',
        company_code: '1375',
        metric: 'ProfitLoss',
        xbrl_tag: 'ProfitLossSummaryOfBusinessResults',
        value: 2989000000,
        unit: 'JPY',
        fiscal_year: 2022,
        consolidated: true,
    },
]

describe('選択中銘柄の財務指標を表示する', () => {
    it('銘柄が未選択のときはプレースホルダーを表示する', () => {
        render(<MetricsPanel selection={null} onFetchMetrics={vi.fn()} />)

        expect(screen.getByTestId('metrics-panel-placeholder')).toBeInTheDocument()
    })

    it('銘柄選択時にonFetchMetricsを証券コードで呼び出す', () => {
        const onFetchMetrics = vi.fn().mockResolvedValue([])

        render(
            <MetricsPanel
                selection={{ code: '1375', name: 'テスト株式会社' }}
                onFetchMetrics={onFetchMetrics}
            />,
        )

        expect(onFetchMetrics).toHaveBeenCalledWith('1375')
    })

    it('取得結果が空のときは空状態を表示する', async () => {
        render(
            <MetricsPanel
                selection={{ code: '1375', name: 'テスト株式会社' }}
                onFetchMetrics={vi.fn().mockResolvedValue([])}
            />,
        )

        expect(await screen.findByTestId('metrics-panel-empty')).toBeInTheDocument()
    })

    it('指標×決算年度のピボット表を表示する', async () => {
        render(
            <MetricsPanel
                selection={{ code: '1375', name: 'テスト株式会社' }}
                onFetchMetrics={vi.fn().mockResolvedValue(mockMetrics)}
            />,
        )

        await waitFor(() => expect(screen.getByTestId('metrics-panel-table')).toBeInTheDocument())

        expect(screen.getByTestId('metrics-row-Revenue')).toBeInTheDocument()
        expect(screen.getByTestId('metrics-row-ProfitLoss')).toBeInTheDocument()
        expect(screen.getByText('2022')).toBeInTheDocument()
        expect(screen.getByText('2026')).toBeInTheDocument()
        // ProfitLossは2026年度のデータが無いので「—」になる
        const profitRow = screen.getByTestId('metrics-row-ProfitLoss')
        expect(profitRow.textContent).toContain('—')
    })
})
