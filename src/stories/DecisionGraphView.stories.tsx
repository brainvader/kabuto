import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, fn, userEvent, within } from 'storybook/test'

import { DecisionGraphView } from '@/components/DecisionGraphView'
import type { DecisionSession, Decision, ConsideredEdge } from '@/lib/decisions'

// decision-flow.html（2026/08/15/007.md）と同じ INPEX/WTI の例をモックデータとして使う。
const sessions: Record<string, DecisionSession> = {
    INPEX_buy_decision: {
        question: 'INPEXを買うべきか？',
        company: 'company:1605',
        status: 'resolved',
        selection_mode: 'composite',
        conclusion:
            'マクロ感応度（WTI原油価格と有意な相関）・財務健全性（増収増益基調）・事業リスク（目立った悪化なし）の3要因を総合し、買い判断を支持する材料が揃っている。',
        action_taken: 'bought',
    },
    INPEX_external_factors: {
        question: 'INPEXの株価に影響する外部要因は何か？',
        company: 'company:1605',
        status: 'resolved',
        selection_mode: 'exclusive',
        conclusion: 'WTI原油価格との相関が有意（r=0.82, lag 3日）。為替・金利は有意な相関が見られなかった。',
        action_taken: 'none',
    },
}

const decisions: Record<string, Decision> = {
    INPEX_WTI_2026Q3: {
        hypothesis: 'company:1605 AFFECTED_BY macro:WTI',
        method: 'pearson_252d_rolling',
        correlation: 0.82,
        confirmed: true,
        lag_days: 3,
        based_on_doc_ids: ['S100Y9AH'],
    },
    INPEX_USDJPY_2026Q3: {
        hypothesis: 'company:1605 AFFECTED_BY macro:USDJPY',
        method: 'pearson_252d_rolling',
        correlation: 0.12,
        confirmed: false,
        lag_days: null,
        based_on_doc_ids: ['S100Y9AH'],
    },
}

const edges: Record<string, ConsideredEdge[]> = {
    INPEX_buy_decision: [
        { target: 'decision_session:INPEX_external_factors', selected: true, rejection_reason: null },
    ],
    INPEX_external_factors: [
        { target: 'decision:INPEX_WTI_2026Q3', selected: true, rejection_reason: null },
        { target: 'decision:INPEX_USDJPY_2026Q3', selected: false, rejection_reason: '低相関 r=0.12' },
    ],
}

async function mockFetchSession(id: string): Promise<DecisionSession | null> {
    return sessions[id] ?? null
}
async function mockFetchDecision(id: string): Promise<Decision | null> {
    return decisions[id] ?? null
}
async function mockFetchConsidered(id: string): Promise<ConsideredEdge[]> {
    return edges[id] ?? []
}

const meta: Meta<typeof DecisionGraphView> = {
    component: DecisionGraphView,
    decorators: [
        (Story) => (
            <div style={{ width: '100vw', height: '100vh', background: '#09090b' }}>
                <Story />
            </div>
        ),
    ],
}
export default meta

type Story = StoryObj<typeof DecisionGraphView>

export const RootOnly: Story = {
    args: {
        rootSessionId: 'INPEX_buy_decision',
        onFetchSession: fn(mockFetchSession),
        onFetchDecision: fn(mockFetchDecision),
        onFetchConsidered: fn(mockFetchConsidered),
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)
        await expect(await canvas.findByText('INPEXを買うべきか？')).toBeInTheDocument()
    },
}

export const ExpandedWithMixedOutcomes: Story = {
    args: {
        rootSessionId: 'INPEX_buy_decision',
        onFetchSession: fn(mockFetchSession),
        onFetchDecision: fn(mockFetchDecision),
        onFetchConsidered: fn(mockFetchConsidered),
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)

        const root = await canvas.findByText('INPEXを買うべきか？')
        await userEvent.click(root)

        const factors = await canvas.findByText('INPEXの株価に影響する外部要因は何か？')
        await userEvent.click(factors)

        await expect(await canvas.findByText('company:1605 AFFECTED_BY macro:WTI')).toBeInTheDocument()
        await expect(await canvas.findByText('✓ 採用')).toBeInTheDocument()
        await expect(await canvas.findByText('✗ 却下')).toBeInTheDocument()
        await expect(await canvas.findByText('低相関 r=0.12')).toBeInTheDocument()

        // WTI decision をクリックするとJSONパネルに相関係数が表示される
        await userEvent.click(await canvas.findByText('company:1605 AFFECTED_BY macro:WTI'))
        const jsonPanel = canvas.getByTestId('decision-json-output')
        await expect(jsonPanel).toHaveTextContent('0.82')
    },
}
