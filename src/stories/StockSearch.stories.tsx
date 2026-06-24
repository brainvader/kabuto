import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { fn } from 'storybook/test'

import { StockSearch } from '@components/StockSearch'
import { useStockSearch } from '@hooks/useStockSearch'

type UseStockSearchReturn = ReturnType<typeof useStockSearch>

const mockResults = [
    { code: '1605', name: 'INPEX CORPORATION', market: 'プライム', sector: '鉱業' },
    { code: '5020', name: 'ENEOSホールディングス', market: 'プライム', sector: '石油・石炭製品' },
    { code: '7203', name: 'トヨタ自動車', market: 'プライム', sector: '輸送用機器' },
    { code: '6758', name: 'ソニーグループ', market: 'プライム', sector: '電気機器' },
    { code: '9984', name: 'ソフトバンクグループ', market: 'プライム', sector: '情報・通信業' },
]

const makeHook = (overrides: Partial<UseStockSearchReturn> = {}) => (): UseStockSearchReturn => ({
    results: mockResults,
    query: '',
    market: null,
    sector: null,
    isLoading: false,
    setQuery: fn(),
    setMarket: fn(),
    setSector: fn(),
    ...overrides,
})

const meta: Meta<typeof StockSearch> = {
    component: StockSearch,
    parameters: {
        layout: 'fullscreen',
    },
    decorators: [
        (Story) => (
            <div style={{ width: "300px", height: "100vh", margin: "0 auto" }}>
                <Story />
            </div>
        ),
    ],
    args: {
        onSelect: fn(),
    },
}

export default meta
type Story = StoryObj<typeof StockSearch>

export const Default: Story = {
    args: { useSearch: makeHook() },
}

export const WithSelection: Story = {
    args: {
        useSearch: makeHook(),
        selectedCode: '1605',
    },
}

export const Loading: Story = {
    args: {
        useSearch: makeHook({ isLoading: true, results: [] }),
    },
}

export const Empty: Story = {
    args: {
        useSearch: makeHook({ results: [], query: 'zzz' }),
    },
}

export const MarketFilter: Story = {
    args: { useSearch: makeHook() },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement)
        await userEvent.click(canvas.getByRole('button', { name: 'JPX' }))
        await expect(args.onSelect).not.toHaveBeenCalled()
    },
}

export const SelectItem: Story = {
    args: { useSearch: makeHook() },
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement)
        await userEvent.click(canvas.getByText('INPEX CORPORATION'))
        await expect(args.onSelect).toHaveBeenCalledWith({
            code: '1605',
            name: 'INPEX CORPORATION',
        })
    },
}