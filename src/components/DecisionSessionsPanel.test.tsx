import React from 'react'
import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { DecisionSessionsPanel } from './DecisionSessionsPanel'
import type { DecisionSessionSummary, DecisionSession } from '@/lib/decisions'

// DecisionSessionsPanel は内部で DecisionGraphView（ReactFlow使用）を描画するためモックする
vi.mock('@xyflow/react', () => ({
    ReactFlow: ({ nodes }: { nodes: { id: string; data: { label: string } }[] }) =>
        React.createElement(
            'div',
            { 'data-testid': 'react-flow' },
            nodes.map((n) => React.createElement('div', { key: n.id }, n.data.label)),
        ),
    Background: () => null,
    BackgroundVariant: { Dots: 'dots' },
}))

const summaries: DecisionSessionSummary[] = [
    {
        id: 'decision_session:a',
        question: 'INPEXを買うべきか？',
        company: 'company:1605',
        status: 'resolved',
        selection_mode: 'composite',
        conclusion: '結論A',
        action_taken: 'bought',
    },
    {
        id: 'decision_session:b',
        question: 'トヨタは割高か？',
        company: 'company:7203',
        status: 'exploring',
        selection_mode: 'exclusive',
        conclusion: null,
        action_taken: 'none',
    },
]

const sessionDetail: DecisionSession = {
    question: 'INPEXを買うべきか？',
    company: 'company:1605',
    status: 'resolved',
    selection_mode: 'composite',
    conclusion: '結論A',
    action_taken: 'bought',
}

describe('セッション一覧を取得して表示する', () => {
    it('マウント時に onListSessions を呼び、質問文を一覧表示する', async () => {
        const onListSessions = vi.fn(async () => summaries)
        render(
            <DecisionSessionsPanel
                onListSessions={onListSessions}
                onFetchSession={vi.fn()}
                onFetchDecision={vi.fn()}
                onFetchConsidered={vi.fn()}
            />,
        )

        await waitFor(() => expect(onListSessions).toHaveBeenCalled())
        expect(await screen.findByText('INPEXを買うべきか？')).toBeInTheDocument()
        expect(await screen.findByText('トヨタは割高か？')).toBeInTheDocument()
    })

    it('セッションが0件のときは空状態を表示する', async () => {
        const onListSessions = vi.fn(async () => [])
        render(
            <DecisionSessionsPanel
                onListSessions={onListSessions}
                onFetchSession={vi.fn()}
                onFetchDecision={vi.fn()}
                onFetchConsidered={vi.fn()}
            />,
        )

        expect(await screen.findByTestId('decision-session-empty')).toBeInTheDocument()
    })
})

describe('セッションを選択する', () => {
    it('初期状態ではプレースホルダーを表示する', async () => {
        const onListSessions = vi.fn(async () => summaries)
        render(
            <DecisionSessionsPanel
                onListSessions={onListSessions}
                onFetchSession={vi.fn()}
                onFetchDecision={vi.fn()}
                onFetchConsidered={vi.fn()}
            />,
        )

        expect(await screen.findByTestId('decision-graph-placeholder')).toBeInTheDocument()
    })

    it('一覧のアイテムをクリックするとそのセッションのグラフ取得が始まる', async () => {
        const onListSessions = vi.fn(async () => summaries)
        const onFetchSession = vi.fn(async () => sessionDetail)
        render(
            <DecisionSessionsPanel
                onListSessions={onListSessions}
                onFetchSession={onFetchSession}
                onFetchDecision={vi.fn()}
                onFetchConsidered={vi.fn(async () => [])}
            />,
        )

        await screen.findByTestId('decision-session-item-decision_session:a')
        await userEvent.click(screen.getByTestId('decision-session-item-decision_session:a'))

        await waitFor(() => expect(onFetchSession).toHaveBeenCalledWith('decision_session:a'))
        expect(screen.queryByTestId('decision-graph-placeholder')).not.toBeInTheDocument()
    })
})
