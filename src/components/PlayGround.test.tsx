import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { PlayGround } from '@/components/PlayGround'

describe('タブ切替', () => {
    it('初期表示は GRAPH タブがアクティブである', () => {
        render(<PlayGround />)
        expect(screen.getByRole('tab', { name: 'GRAPH' })).toHaveAttribute('aria-selected', 'true')
        expect(screen.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'false')
    })

    it('GRAPH タブ選択時はキャンバスエリアが表示される', () => {
        render(<PlayGround />)
        expect(screen.getByTestId('playground-canvas')).toBeInTheDocument()
        expect(screen.queryByTestId('playground-result')).not.toBeInTheDocument()
    })

    it('RESULT タブをクリックすると結果エリアが表示される', async () => {
        render(<PlayGround />)
        await userEvent.click(screen.getByRole('tab', { name: 'RESULT' }))
        expect(screen.getByTestId('playground-result')).toBeInTheDocument()
        expect(screen.queryByTestId('playground-canvas')).not.toBeInTheDocument()
    })

    it('RESULT タブがアクティブになる', async () => {
        render(<PlayGround />)
        await userEvent.click(screen.getByRole('tab', { name: 'RESULT' }))
        expect(screen.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'true')
        expect(screen.getByRole('tab', { name: 'GRAPH' })).toHaveAttribute('aria-selected', 'false')
    })
})