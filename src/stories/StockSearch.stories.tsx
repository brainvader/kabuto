import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { fn } from 'storybook/test'

import { StockSearch } from '@components/StockSearch'

const meta: Meta<typeof StockSearch> = {
    component: StockSearch,
    parameters: {
        layout: 'fullscreen',
    },
    decorators: [
        (Story) => (
            <div style={{ display: 'flex', justifyContent: 'center', height: '100vh' }}>
                <div style={{ width: '300px', height: '100vh' }}>
                    <Story />
                </div>
            </div>
        ),
    ],
    args: {
        onSelect: fn(),
    },
}

export default meta
type Story = StoryObj<typeof StockSearch>

export const Default: Story = {}

export const WithSelection: Story = {
    args: { selectedCode: '1605' },
}

export const MarketFilter: Story = {
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement)
        await userEvent.click(canvas.getByRole('button', { name: 'JPX' }))
        await expect(args.onSelect).not.toHaveBeenCalled()
    },
}

export const SelectItem: Story = {
    play: async ({ canvasElement, args }) => {
        const canvas = within(canvasElement)
        // 検索を実行して結果を表示してから選択
        await userEvent.type(canvas.getByRole('searchbox'), 'INPEX')
        await userEvent.click(canvas.getByText('INPEX CORPORATION'))
        await expect(args.onSelect).toHaveBeenCalledWith({
            code: '1605',
            name: 'INPEX CORPORATION',
        })
    },
}