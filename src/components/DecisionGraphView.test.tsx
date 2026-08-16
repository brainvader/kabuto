import React from 'react'
import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { DecisionGraphView } from './DecisionGraphView'
import type { DecisionSession, Decision, ConsideredEdge } from '@/lib/decisions'

// ReactFlow はブラウザ依存のためモックする。各ノードは data.label / data.onClick を持つ
// 共通シェイプなので、クリック可能な div として描画すれば足りる。
vi.mock('@xyflow/react', () => ({
    ReactFlow: ({ nodes, edges }: { nodes: { id: string; data: { label: string; onClick: () => void } }[]; edges: { id: string }[] }) =>
        React.createElement(
            'div',
            { 'data-testid': 'react-flow', 'data-node-count': nodes.length, 'data-edge-count': edges.length },
            nodes.map((n) =>
                React.createElement(
                    'div',
                    { key: n.id, 'data-testid': `flow-node-${n.id}`, onClick: () => n.data.onClick() },
                    n.data.label,
                ),
            ),
        ),
    Background: () => null,
    BackgroundVariant: { Dots: 'dots' },
}))

const rootSession: DecisionSession = {
    question: 'INPEXを買うべきか？',
    company: 'company:1605',
    status: 'resolved',
    selection_mode: 'composite',
    conclusion: '結論テキスト',
    action_taken: 'bought',
}

const macroSession: DecisionSession = {
    question: 'マクロ感応度',
    company: 'company:1605',
    status: 'resolved',
    selection_mode: 'exclusive',
    conclusion: 'WTIとの相関が有意',
    action_taken: 'none',
}

const wtiDecision: Decision = {
    hypothesis: 'company:1605 AFFECTED_BY macro:WTI',
    method: 'pearson_252d_rolling',
    correlation: 0.82,
    confirmed: true,
    lag_days: 3,
    based_on_doc_ids: ['S100Y9AH'],
}

const rootEdges: ConsideredEdge[] = [{ target: 'decision_session:macro', selected: true, rejection_reason: null }]
const macroEdges: ConsideredEdge[] = [{ target: 'decision:wti', selected: true, rejection_reason: null }]

function makeFetchers() {
    const onFetchSession = vi.fn(async (id: string): Promise<DecisionSession | null> => {
        if (id === 'root') return rootSession
        if (id === 'macro') return macroSession
        return null
    })
    const onFetchDecision = vi.fn(async (id: string): Promise<Decision | null> => (id === 'wti' ? wtiDecision : null))
    const onFetchConsidered = vi.fn(async (id: string): Promise<ConsideredEdge[]> => {
        if (id === 'root') return rootEdges
        if (id === 'macro') return macroEdges
        return []
    })
    return { onFetchSession, onFetchDecision, onFetchConsidered }
}

describe('マウント時にルートセッションを取得する', () => {
    it('rootSessionId を onFetchSession に渡し、1ノードとして表示する', async () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )

        await waitFor(() => expect(onFetchSession).toHaveBeenCalledWith('root'))
        await waitFor(() => expect(screen.getByTestId('react-flow')).toHaveAttribute('data-node-count', '1'))
    })
})

describe('セッションノードをクリックして展開する', () => {
    it('CONSIDEREDエッジと対象の詳細を取得し子ノードが追加される', async () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )

        await waitFor(() => expect(screen.getByTestId('flow-node-root')).toBeInTheDocument())
        await userEvent.click(screen.getByTestId('flow-node-root'))

        await waitFor(() => expect(onFetchConsidered).toHaveBeenCalledWith('root'))
        await waitFor(() => expect(onFetchSession).toHaveBeenCalledWith('macro'))
        await waitFor(() => expect(screen.getByTestId('react-flow')).toHaveAttribute('data-node-count', '2'))
    })

    it('展開済みのセッションを再度クリックすると折りたたまれ子ノードが消える', async () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )

        await waitFor(() => screen.getByTestId('flow-node-root'))
        await userEvent.click(screen.getByTestId('flow-node-root'))
        await waitFor(() => expect(screen.getByTestId('react-flow')).toHaveAttribute('data-node-count', '2'))

        await userEvent.click(screen.getByTestId('flow-node-root'))
        await waitFor(() => expect(screen.getByTestId('react-flow')).toHaveAttribute('data-node-count', '1'))

        // 再展開時はキャッシュ済みなので再フェッチしない
        await userEvent.click(screen.getByTestId('flow-node-root'))
        await waitFor(() => expect(screen.getByTestId('react-flow')).toHaveAttribute('data-node-count', '2'))
        expect(onFetchConsidered).toHaveBeenCalledTimes(1)
    })

    it('孫ノード(decision)まで展開すると3ノードになる', async () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )

        await waitFor(() => screen.getByTestId('flow-node-root'))
        await userEvent.click(screen.getByTestId('flow-node-root'))
        await waitFor(() => screen.getByTestId('flow-node-macro'))

        await userEvent.click(screen.getByTestId('flow-node-macro'))
        await waitFor(() => expect(onFetchDecision).toHaveBeenCalledWith('wti'))
        await waitFor(() => expect(screen.getByTestId('react-flow')).toHaveAttribute('data-node-count', '3'))
    })
})

describe('ノードをクリックするとJSONパネルに表示される', () => {
    it('初期状態ではプレースホルダーを表示する', () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )
        expect(screen.getByTestId('decision-json-panel')).toHaveTextContent('ノードをクリックしてください')
    })

    it('セッションノードをクリックすると質問文がJSONとして表示される', async () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )

        await waitFor(() => screen.getByTestId('flow-node-root'))
        await userEvent.click(screen.getByTestId('flow-node-root'))

        await waitFor(() => expect(screen.getByTestId('decision-json-output')).toHaveTextContent('INPEXを買うべきか？'))
    })

    it('decisionノードをクリックすると仮説と相関係数がJSONとして表示される', async () => {
        const { onFetchSession, onFetchDecision, onFetchConsidered } = makeFetchers()
        render(
            <DecisionGraphView
                rootSessionId="root"
                onFetchSession={onFetchSession}
                onFetchDecision={onFetchDecision}
                onFetchConsidered={onFetchConsidered}
            />,
        )

        await waitFor(() => screen.getByTestId('flow-node-root'))
        await userEvent.click(screen.getByTestId('flow-node-root'))
        await waitFor(() => screen.getByTestId('flow-node-macro'))
        await userEvent.click(screen.getByTestId('flow-node-macro'))
        await waitFor(() => screen.getByTestId('flow-node-wti'))

        await userEvent.click(screen.getByTestId('flow-node-wti'))
        await waitFor(() => expect(screen.getByTestId('decision-json-output')).toHaveTextContent('0.82'))
    })
})
