import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'

import { PlayGround } from '../components/PlayGround'

const meta: Meta<typeof PlayGround> = {
    component: PlayGround,
}
export default meta

type Story = StoryObj<typeof PlayGround>

export const Default: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)

        // 初期状態: GRAPH タブがアクティブ
        await expect(canvas.getByRole('tab', { name: 'GRAPH' })).toHaveAttribute('aria-selected', 'true')
        await expect(canvas.getByTestId('playground-canvas')).toBeInTheDocument()

        // RESULT タブに切り替え
        await userEvent.click(canvas.getByRole('tab', { name: 'RESULT' }))
        await expect(canvas.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'true')
        await expect(canvas.getByTestId('playground-result')).toBeInTheDocument()

        // GRAPH タブに戻る
        await userEvent.click(canvas.getByRole('tab', { name: 'GRAPH' }))
        await expect(canvas.getByTestId('playground-canvas')).toBeInTheDocument()
    },
}