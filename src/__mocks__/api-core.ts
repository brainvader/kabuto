// Tauri IPC モック — Vitest / Storybook 共通

const mockStocks = [
    { code: '1605', name: 'INPEX CORPORATION', market: 'プライム', sector: '鉱業' },
    { code: '5020', name: 'ENEOSホールディングス', market: 'プライム', sector: '石油・石炭製品' },
    { code: '7203', name: 'トヨタ自動車', market: 'プライム', sector: '輸送用機器' },
    { code: '6758', name: 'ソニーグループ', market: 'プライム', sector: '電気機器' },
    { code: '9984', name: 'ソフトバンクグループ', market: 'プライム', sector: '情報・通信業' },
]

export const invoke = async (cmd: string): Promise<unknown> => {
    if (cmd === 'search_stocks') {
        return mockStocks
    }
    return null
}