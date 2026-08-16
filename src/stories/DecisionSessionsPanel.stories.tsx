import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, fn, userEvent, within } from 'storybook/test'

import { DecisionSessionsPanel } from '@/components/DecisionSessionsPanel'
import type { DecisionSessionSummary, DecisionSession, Decision, ConsideredEdge } from '@/lib/decisions'

const summaries: DecisionSessionSummary[] = [
    {
        id: 'INPEX_buy_decision',
        question: 'INPEXを買うべきか？',
        company: 'company:1605',
        status: 'resolved',
        selection_mode: 'composite',
        conclusion: 'マクロ感応度・財務健全性・事業リスクの3要因を総合し買い判断を支持',
        action_taken: 'bought',
    },
    {
        id: 'toyota_valuation',
        question: 'トヨタは割高か？',
        company: 'company:7203',
        status: 'exploring',
        selection_mode: 'exclusive',
        conclusion: null,
        action_taken: 'none',
    },
]

const sessions: Record<string, DecisionSession> = {
    INPEX_buy_decision: summaries[0],
}

async function mockFetchSession(id: string): Promise<DecisionSession | null> {
    return sessions[id] ?? null
}
async function mockFetchDecision(): Promise<Decision | null> {
    return null
}
async function mockFetchConsidered(): Promise<ConsideredEdge[]> {
    return []
}

const meta: Meta<typeof DecisionSessionsPanel> = {
    component: DecisionSessionsPanel,
    decorators: [
        (Story) => (
            <div style={{ width: '100vw', height: '100vh', background: '#09090b' }}>
                <Story />
            </div>
        ),
    ],
}
export default meta

type Story = StoryObj<typeof DecisionSessionsPanel>

export const Empty: Story = {
    args: {
        onListSessions: fn(async () => []),
        onFetchSession: fn(mockFetchSession),
        onFetchDecision: fn(mockFetchDecision),
        onFetchConsidered: fn(mockFetchConsidered),
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)
        await expect(await canvas.findByTestId('decision-session-empty')).toBeInTheDocument()
    },
}

export const SelectSession: Story = {
    args: {
        onListSessions: fn(async () => summaries),
        onFetchSession: fn(mockFetchSession),
        onFetchDecision: fn(mockFetchDecision),
        onFetchConsidered: fn(mockFetchConsidered),
    },
    play: async ({ canvasElement }) => {
        const canvas = within(canvasElement)

        await expect(await canvas.findByText('INPEXを買うべきか？')).toBeInTheDocument()
        await expect(await canvas.findByTestId('decision-graph-placeholder')).toBeInTheDocument()

        await userEvent.click(await canvas.findByText('INPEXを買うべきか？'))

        await expect(canvas.queryByTestId('decision-graph-placeholder')).not.toBeInTheDocument()
    },
}
