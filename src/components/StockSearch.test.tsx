import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { StockSearch } from './StockSearch'
import { useStockSearch } from '../hooks/useStockSearch'

// ── モック ──────────────────────────────────────────────────
// useStockSearch の戻り値型から動的にモックを生成する
type UseStockSearchReturn = ReturnType<typeof useStockSearch>

vi.mock('../hooks/useStockSearch')
const mockUseStockSearch = vi.mocked(useStockSearch)

const mockResults = [
    { code: '1605', name: 'INPEX CORPORATION', market: 'プライム', sector: '鉱業' },
    { code: '5020', name: 'ENEOSホールディングス', market: 'プライム', sector: '石油・石炭製品' },
    { code: '7203', name: 'トヨタ自動車', market: 'プライム', sector: '輸送用機器' },
]

const defaultHookReturn: UseStockSearchReturn = {
    results: mockResults,
    query: '',
    market: null,
    sector: null,
    isLoading: false,
    setQuery: vi.fn(),
    setMarket: vi.fn(),
    setSector: vi.fn(),
}

// ── テスト ──────────────────────────────────────────────────

describe('銘柄を検索・選択する', () => {
    it('検索ボックスにテキストを入力すると setQuery が呼ばれる', () => {
        const setQuery = vi.fn()
        mockUseStockSearch.mockReturnValue({ ...defaultHookReturn, setQuery })

        render(<StockSearch onSelect={vi.fn()} />)

        fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'INPEX' } })
        expect(setQuery).toHaveBeenCalledWith('INPEX')
    })

    it('市場フィルターをクリックすると setMarket が呼ばれる', async () => {
        const setMarket = vi.fn()
        mockUseStockSearch.mockReturnValue({ ...defaultHookReturn, setMarket })

        render(<StockSearch onSelect={vi.fn()} />)

        await userEvent.click(screen.getByRole('button', { name: 'JPX' }))
        expect(setMarket).toHaveBeenCalledWith('JPX')
    })

    it('結果リストに銘柄が表示される', () => {
        mockUseStockSearch.mockReturnValue(defaultHookReturn)

        render(<StockSearch onSelect={vi.fn()} />)

        expect(screen.getByText('1605')).toBeInTheDocument()
        expect(screen.getByText('INPEX CORPORATION')).toBeInTheDocument()
        expect(screen.getByText('7203')).toBeInTheDocument()
    })

    it('結果アイテムをクリックすると onSelect が code と name で呼ばれる', async () => {
        mockUseStockSearch.mockReturnValue(defaultHookReturn)
        const onSelect = vi.fn()

        render(<StockSearch onSelect={onSelect} />)

        await userEvent.click(screen.getByText('INPEX CORPORATION'))
        expect(onSelect).toHaveBeenCalledWith({ code: '1605', name: 'INPEX CORPORATION' })
    })

    it('selectedCode と一致するアイテムは selected スタイルで表示される', () => {
        mockUseStockSearch.mockReturnValue(defaultHookReturn)

        render(<StockSearch onSelect={vi.fn()} selectedCode="1605" />)

        const item = screen.getByTestId('stock-item-1605')
        expect(item).toHaveAttribute('data-selected', 'true')
    })

    it('isLoading が true のときローディング表示になる', () => {
        mockUseStockSearch.mockReturnValue({ ...defaultHookReturn, isLoading: true, results: [] })

        render(<StockSearch onSelect={vi.fn()} />)

        expect(screen.getByTestId('stock-search-loading')).toBeInTheDocument()
    })
})