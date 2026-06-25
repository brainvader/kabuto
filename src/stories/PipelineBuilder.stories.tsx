import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'

import { PipelineBuilder } from '../components/PipelineBuilder'

const meta: Meta<typeof PipelineBuilder> = {
    component: PipelineBuilder,
}
export default meta

type Story = StoryObj<typeof PipelineBuilder>

export const Default: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)

        // 初期状態: BUILDER タブがアクティブ
        await expect(canvas.getByRole('tab', { name: 'BUILDER' })).toHaveAttribute('aria-selected', 'true')
        await expect(canvas.getByTestId('pipeline-canvas')).toBeInTheDocument()

        // RESULT タブに切り替え
        await userEvent.click(canvas.getByRole('tab', { name: 'RESULT' }))
        await expect(canvas.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'true')
        await expect(canvas.getByTestId('pipeline-result')).toBeInTheDocument()

        // BUILDER タブに戻る
        await userEvent.click(canvas.getByRole('tab', { name: 'BUILDER' }))
        await expect(canvas.getByTestId('pipeline-canvas')).toBeInTheDocument()
    },
}