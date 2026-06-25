import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { PipelineBuilder } from '@/components/PipelineBuilder'

describe('タブ切替', () => {
    it('初期表示は BUILDER タブがアクティブである', () => {
        render(<PipelineBuilder />)
        expect(screen.getByRole('tab', { name: 'BUILDER' })).toHaveAttribute('aria-selected', 'true')
        expect(screen.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'false')
    })

    it('BUILDER タブ選択時はキャンバスエリアが表示される', () => {
        render(<PipelineBuilder />)
        expect(screen.getByTestId('pipeline-canvas')).toBeInTheDocument()
        expect(screen.queryByTestId('pipeline-result')).not.toBeInTheDocument()
    })

    it('RESULT タブをクリックすると結果エリアが表示される', async () => {
        render(<PipelineBuilder />)
        await userEvent.click(screen.getByRole('tab', { name: 'RESULT' }))
        expect(screen.getByTestId('pipeline-result')).toBeInTheDocument()
        expect(screen.queryByTestId('pipeline-canvas')).not.toBeInTheDocument()
    })

    it('RESULT タブがアクティブになる', async () => {
        render(<PipelineBuilder />)
        await userEvent.click(screen.getByRole('tab', { name: 'RESULT' }))
        expect(screen.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'true')
        expect(screen.getByRole('tab', { name: 'BUILDER' })).toHaveAttribute('aria-selected', 'false')
    })
})