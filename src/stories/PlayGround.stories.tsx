import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'

import { PlayGround } from '@components/PlayGround'
import { useKabutoStore } from '@/store/useKabutoStore'

const meta: Meta<typeof PlayGround> = {
    component: PlayGround,
    beforeEach: () => {
        useKabutoStore.setState({ selection: null, pipeline: { status: 'idle', activeId: null } })
    },
}
export default meta

type Story = StoryObj<typeof PlayGround>

// ── 選択なし ──────────────────────────────────────────────────────────────────

export const Empty: Story = {
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)
        await expect(canvas.getByTestId('playground-empty')).toBeInTheDocument()
    },
}

// ── 選択あり ──────────────────────────────────────────────────────────────────

export const WithSelection: Story = {
    beforeEach: () => {
        useKabutoStore.setState({
            selection: { code: '1605', name: 'INPEX' },
            pipeline: { status: 'idle', activeId: null },
        })
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)
        await expect(canvas.getByTestId('node-company')).toBeInTheDocument()
        await expect(canvas.getByText('1605')).toBeInTheDocument()
        await expect(canvas.getByText('INPEX')).toBeInTheDocument()
    },
}

// ── タブ切替 ──────────────────────────────────────────────────────────────────

export const TabSwitch: Story = {
    beforeEach: () => {
        useKabutoStore.setState({
            selection: { code: '7203', name: 'トヨタ自動車' },
            pipeline: { status: 'idle', activeId: null },
        })
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)

        await expect(canvas.getByRole('tab', { name: 'GRAPH' })).toHaveAttribute('aria-selected', 'true')
        await expect(canvas.getByTestId('node-company')).toBeInTheDocument()

        await userEvent.click(canvas.getByRole('tab', { name: 'RESULT' }))
        await expect(canvas.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'true')
        await expect(canvas.getByTestId('playground-result')).toBeInTheDocument()
    },
}