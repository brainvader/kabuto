import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { PlayGround } from './PlayGround'
import { useKabutoStore } from '../store/useKabutoStore'

beforeEach(() => {
    useKabutoStore.setState({ selection: null, pipeline: { status: 'idle', activeId: null } })
})

// ── タブ切替 ─────────────────────────────────────────────────────────────────

describe('タブ切替', () => {
    it('初期表示は GRAPH タブがアクティブである', () => {
        render(<PlayGround />)
        expect(screen.getByRole('tab', { name: 'GRAPH' })).toHaveAttribute('aria-selected', 'true')
        expect(screen.getByRole('tab', { name: 'RESULT' })).toHaveAttribute('aria-selected', 'false')
    })

    it('RESULT タブをクリックすると結果エリアが表示される', async () => {
        render(<PlayGround />)
        await userEvent.click(screen.getByRole('tab', { name: 'RESULT' }))
        expect(screen.getByTestId('playground-result')).toBeInTheDocument()
        expect(screen.queryByTestId('playground-canvas')).not.toBeInTheDocument()
    })
})

// ── selection が null ─────────────────────────────────────────────────────────

describe('selection が null の場合', () => {
    it('プレースホルダーが表示される', () => {
        render(<PlayGround />)
        expect(screen.getByTestId('playground-empty')).toBeInTheDocument()
    })
})

// ── selection がある場合 ──────────────────────────────────────────────────────

describe('selection がある場合', () => {
    beforeEach(() => {
        useKabutoStore.setState({
            selection: { code: '1605', name: 'INPEX' },
            pipeline: { status: 'idle', activeId: null },
        })
    })

    it('選択銘柄のノードが表示される', () => {
        render(<PlayGround />)
        expect(screen.getByTestId('node-company')).toBeInTheDocument()
    })

    it('証券コードが表示される', () => {
        render(<PlayGround />)
        expect(screen.getByText('1605')).toBeInTheDocument()
    })

    it('銘柄名が表示される', () => {
        render(<PlayGround />)
        expect(screen.getByText('INPEX')).toBeInTheDocument()
    })

    it('selection が変わるとノードが更新される', () => {
        render(<PlayGround />)
        expect(screen.getByText('INPEX')).toBeInTheDocument()

        act(() => {
            useKabutoStore.setState({ selection: { code: '7203', name: 'トヨタ自動車' } })
        })

        expect(screen.getByText('トヨタ自動車')).toBeInTheDocument()
    })
})