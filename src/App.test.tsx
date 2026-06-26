import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import React from 'react'

import App from './App'
import { useKabutoStore } from '@/store/useKabutoStore'

const { mockOnSelect } = vi.hoisted(() => ({ mockOnSelect: vi.fn() }))

vi.mock('./components/StockSearch', () => ({
    StockSearch: (props: { onSelect: (s: { code: string; name: string }) => void }) => {
        mockOnSelect.mockImplementation(props.onSelect)
        return React.createElement('div', { 'data-testid': 'stock-search' })
    },
}))

vi.mock('./components/PlayGround', () => ({
    PlayGround: () => React.createElement('div', { 'data-testid': 'playground' }),
}))

beforeEach(() => {
    useKabutoStore.setState({ selection: null, pipeline: { status: 'idle', activeId: null } })
    mockOnSelect.mockReset()
})

describe('App レイアウト', () => {
    it('StockSearch パネルが表示される', () => {
        render(<App />)
        expect(screen.getByTestId('stock-search')).toBeInTheDocument()
    })

    it('PlayGround パネルが表示される', () => {
        render(<App />)
        expect(screen.getByTestId('playground-panel')).toBeInTheDocument()
    })

    it('Metrics パネルが表示される', () => {
        render(<App />)
        expect(screen.getByTestId('metrics-panel')).toBeInTheDocument()
    })
})

describe('StockSearch → store 連携', () => {
    it('onSelect が呼ばれると selection が更新される', () => {
        render(<App />)
        mockOnSelect({ code: '1605', name: 'INPEX' })
        expect(useKabutoStore.getState().selection).toEqual({ code: '1605', name: 'INPEX' })
    })
})