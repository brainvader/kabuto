import { useState, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'

/** master.parquet の1行に対応する銘柄情報 */
export interface StockItem {
    code: string
    name: string
    market: string
    sector: string
}

/** useStockSearch の戻り値 */
export interface UseStockSearchReturn {
    results: StockItem[]
    query: string
    market: string | null
    sector: string | null
    isLoading: boolean
    setQuery: (query: string) => void
    setMarket: (market: string | null) => void
    setSector: (sector: string | null) => void
}

/**
 * 銘柄検索クエリを管理するフック。
 * Tauri IPC経由で search_stocks コマンドを呼び出し、
 * master.parquet を Polars でフィルタリングする。
 *
 * - query が空のとき invoke を発行しない
 * - 結果は最大 50 件（Rust側 + フロント側で制限）
 */
export function useStockSearch(): UseStockSearchReturn {
    const [results, setResults] = useState<StockItem[]>([])
    const [query, setQueryState] = useState('')
    const [market, setMarketState] = useState<string | null>(null)
    const [sector, setSectorState] = useState<string | null>(null)
    const [isLoading, setIsLoading] = useState(false)

    // 最新のフィルター値を常に参照するためにrefを使う
    const queryRef = useRef(query)
    const marketRef = useRef(market)
    const sectorRef = useRef(sector)

    const search = useCallback(async (q: string, m: string | null, s: string | null) => {
        if (!q.trim()) {
            setResults([])
            return
        }
        setIsLoading(true)
        try {
            const items = await invoke<StockItem[]>('search_stocks', {
                query: q,
                market: m,
                sector: s,
            })
            setResults(items.slice(0, 50))
        } catch (e) {
            console.error('search_stocks failed:', e)
            setResults([])
        } finally {
            setIsLoading(false)
        }
    }, [])

    const setQuery = useCallback((q: string) => {
        queryRef.current = q
        setQueryState(q)
        search(q, marketRef.current, sectorRef.current)
    }, [search])

    const setMarket = useCallback((m: string | null) => {
        marketRef.current = m
        setMarketState(m)
        search(queryRef.current, m, sectorRef.current)
    }, [search])

    const setSector = useCallback((s: string | null) => {
        sectorRef.current = s
        setSectorState(s)
        search(queryRef.current, marketRef.current, s)
    }, [search])

    return { results, query, market, sector, isLoading, setQuery, setMarket, setSector }
}