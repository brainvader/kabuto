import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'

import { useStockSearch } from '@/hooks/useStockSearch'

// ── モック ──────────────────────────────────────────────────
const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: mockInvoke }))

const mockStocks = [
    { code: '1605', name: 'INPEX CORPORATION', market: 'プライム', sector: '鉱業' },
    { code: '5020', name: 'ENEOSホールディングス', market: 'プライム', sector: '石油・石炭製品' },
    { code: '7203', name: 'トヨタ自動車', market: 'プライム', sector: '輸送用機器' },
]

// ── テスト ──────────────────────────────────────────────────

describe('銘柄検索クエリを管理する', () => {
    beforeEach(() => {
        vi.clearAllMocks()
        mockInvoke.mockResolvedValue(mockStocks)
    })

    it('初期状態では query・market・sector が空で results が空配列', () => {
        const { result } = renderHook(() => useStockSearch())

        expect(result.current.query).toBe('')
        expect(result.current.market).toBeNull()
        expect(result.current.sector).toBeNull()
        expect(result.current.results).toEqual([])
    })

    it('setQuery を呼ぶと invoke("search_stocks") が発行される', async () => {
        const { result } = renderHook(() => useStockSearch())

        await act(async () => {
            result.current.setQuery('INPEX')
        })

        expect(mockInvoke).toHaveBeenCalledWith('search_stocks', {
            query: 'INPEX',
            market: null,
            sector: null,
        })
    })

    it('setMarket を呼ぶと market フィルター付きで invoke が発行される', async () => {
        const { result } = renderHook(() => useStockSearch())

        await act(async () => {
            result.current.setQuery('エネルギー')
        })

        await act(async () => {
            result.current.setMarket('プライム')
        })

        expect(mockInvoke).toHaveBeenCalledWith('search_stocks', expect.objectContaining({
            market: 'プライム',
        }))
    })

    it('setSector を呼ぶと sector フィルター付きで invoke が発行される', async () => {
        const { result } = renderHook(() => useStockSearch())

        await act(async () => {
            result.current.setQuery('INPEX')
        })

        await act(async () => {
            result.current.setSector('鉱業')
        })

        expect(mockInvoke).toHaveBeenCalledWith('search_stocks', expect.objectContaining({
            sector: '鉱業',
        }))
    })

    it('結果は最大 50 件に制限される', async () => {
        const manyStocks = Array.from({ length: 100 }, (_, i) => ({
            code: String(i + 1000),
            name: `銘柄${i}`,
            market: 'プライム',
            sector: '情報・通信業',
        }))
        mockInvoke.mockResolvedValue(manyStocks)

        const { result } = renderHook(() => useStockSearch())

        await act(async () => {
            result.current.setQuery('銘柄')
        })

        expect(result.current.results.length).toBeLessThanOrEqual(50)
    })

    it('query が空のとき invoke を発行しない', async () => {
        const { result } = renderHook(() => useStockSearch())

        await act(async () => {
            result.current.setQuery('')
        })

        expect(mockInvoke).not.toHaveBeenCalled()
    })
})