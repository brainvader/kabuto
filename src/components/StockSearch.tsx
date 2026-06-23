import { Search, Loader2 } from 'lucide-react'
import { useStockSearch, type StockItem } from '../hooks/useStockSearch'

const MARKETS = ['ALL', 'JPX', 'NYSE', 'NASDAQ'] as const
const SECTORS = ['全セクター', 'エネルギー', '素材', '情報技術', '金融', '輸送用機器', '電気機器'] as const

/** StockSearch コンポーネントの props */
export interface StockSearchProps {
    /** 銘柄選択時のコールバック */
    onSelect: (item: Pick<StockItem, 'code' | 'name'>) => void
    /** 現在選択中の証券コード */
    selectedCode?: string
    /** Tauri IPC を差し替えるためのフック（テスト用 DI） */
    useSearch?: typeof useStockSearch
}

/**
 * 銘柄検索・選択コンポーネント。
 * master.parquet を Tauri IPC 経由で検索し、結果を一覧表示する。
 */
export function StockSearch({
    onSelect,
    selectedCode,
    useSearch = useStockSearch,
}: StockSearchProps) {
    const { results, query, market, sector, isLoading, setQuery, setMarket, setSector } = useSearch()

    const handleMarketClick = (m: string) => {
        setMarket(m === 'ALL' ? null : m)
    }

    const handleSectorClick = (s: string) => {
        setSector(s === '全セクター' ? null : s)
    }

    return (
        <nav
            id="stock-search"
            className="flex flex-col h-full bg-(--kabuto-card) border border-(--kabuto-card-border) overflow-hidden"
        >
            {/* Panel Header */}
            <div className="flex items-center justify-between px-3.5 py-2 bg-(--kabuto-panel) border-b border-(--kabuto-border) shrink-0">
                <span className="font-mono text-[9px] font-bold tracking-[0.15em] uppercase text-(--kabuto-fg-dim)">
                    01 · Stock Search
                </span>
            </div>

            {/* 検索ボックス */}
            <div className="px-3 py-2.5 bg-(--kabuto-panel) border-b border-(--kabuto-border) shrink-0">
                <div className="relative">
                    <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3 text-(--kabuto-fg-dim)" />
                    <input
                        role="searchbox"
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="コード・社名… 例: 1605, INPEX"
                        className="w-full bg-(--kabuto-input) border border-(--kabuto-border) rounded-sm pl-7 pr-3 py-1.5 font-mono text-[12px] text-(--kabuto-fg) placeholder:text-(--kabuto-fg-subtle) outline-none focus:border-(--kabuto-accent)"
                    />
                </div>
            </div>

            {/* 市場フィルター */}
            <div className="flex gap-1 px-3 py-2 border-b border-(--kabuto-border) flex-wrap shrink-0">
                {MARKETS.map((m) => {
                    const isActive = m === 'ALL' ? market === null : market === m
                    return (
                        <button
                            key={m}
                            onClick={() => handleMarketClick(m)}
                            className={`px-2 py-0.5 rounded-sm font-mono text-[9px] font-semibold tracking-wide border cursor-pointer ${isActive
                                ? 'bg-(--kabuto-accent-dim) text-(--kabuto-accent) border-(--kabuto-accent)'
                                : 'bg-transparent text-(--kabuto-fg-dim) border-(--kabuto-border)'
                                }`}
                        >
                            {m}
                        </button>
                    )
                })}
            </div>

            {/* セクターフィルター */}
            <div className="flex gap-1 px-3 py-2 border-b border-(--kabuto-border) flex-wrap shrink-0">
                {SECTORS.map((s) => {
                    const isActive = s === '全セクター' ? sector === null : sector === s
                    return (
                        <button
                            key={s}
                            onClick={() => handleSectorClick(s)}
                            className={`px-2 py-0.5 rounded-sm font-mono text-[9px] font-semibold tracking-wide border cursor-pointer ${isActive
                                ? 'bg-(--kabuto-accent-dim) text-(--kabuto-accent) border-(--kabuto-accent)'
                                : 'bg-transparent text-(--kabuto-fg-dim) border-(--kabuto-border)'
                                }`}
                        >
                            {s}
                        </button>
                    )
                })}
            </div>

            {/* 検索結果リスト */}
            <div className="flex-1 overflow-y-auto">
                {isLoading ? (
                    <div
                        data-testid="stock-search-loading"
                        className="flex items-center justify-center py-8 gap-2 text-(--kabuto-fg-dim)"
                    >
                        <Loader2 className="size-4 animate-spin" />
                        <span className="font-mono text-[11px]">検索中...</span>
                    </div>
                ) : (
                    <div className="flex flex-col">
                        {results.map((item) => (
                            <div
                                key={item.code}
                                data-testid={`stock-item-${item.code}`}
                                data-selected={item.code === selectedCode ? 'true' : 'false'}
                                onClick={() => onSelect({ code: item.code, name: item.name })}
                                className={`flex items-center justify-between px-3 py-2 border-b border-(--kabuto-border) cursor-pointer gap-2 hover:bg-(--kabuto-muted) ${item.code === selectedCode
                                    ? 'bg-(--kabuto-accent-dim) border-l-2 border-l-(--kabuto-accent)'
                                    : ''
                                    }`}
                            >
                                <div className="flex flex-col gap-0.5 min-w-0">
                                    <span className="font-mono text-[11px] font-bold text-(--kabuto-accent)">
                                        {item.code}
                                    </span>
                                    <span className="text-[11px] text-(--kabuto-fg-dim) truncate max-w-40">
                                        {item.name}
                                    </span>
                                </div>
                                <div className="flex flex-col items-end gap-0.5 shrink-0">
                                    <span className="font-mono text-[9px] text-(--kabuto-fg-dim)">
                                        {item.market}
                                    </span>
                                    <span className="font-mono text-[9px] text-(--kabuto-fg-subtle)">
                                        {item.sector}
                                    </span>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </nav>
    )
}