import { useEffect, useState } from 'react'

import {
    listDecisionSessions,
    fetchDecisionSession,
    fetchDecision,
    fetchConsidered,
    type DecisionSessionSummary,
    type DecisionSession,
    type Decision,
    type ConsideredEdge,
} from '@/lib/decisions'
import { describeActionTaken } from '@/domain/decisionSession'
import { DecisionGraphView } from '@/components/DecisionGraphView'

export interface DecisionSessionsPanelProps {
    onListSessions?: () => Promise<DecisionSessionSummary[]>
    onFetchSession?: (id: string) => Promise<DecisionSession | null>
    onFetchDecision?: (id: string) => Promise<Decision | null>
    onFetchConsidered?: (sessionId: string) => Promise<ConsideredEdge[]>
}

/**
 * 記録済みの decision_session を一覧し、選択したセッションを DecisionGraphView で表示する。
 * PlayGround の DECISION タブから使う（2026/08/15/007.md Step 7 の App 組み込み）。
 */
export function DecisionSessionsPanel({
    onListSessions = listDecisionSessions,
    onFetchSession = fetchDecisionSession,
    onFetchDecision = fetchDecision,
    onFetchConsidered = fetchConsidered,
}: DecisionSessionsPanelProps) {
    const [sessions, setSessions] = useState<DecisionSessionSummary[]>([])
    const [selectedId, setSelectedId] = useState<string | null>(null)

    useEffect(() => {
        let cancelled = false
        onListSessions().then((list) => {
            if (!cancelled) setSessions(list)
        })
        return () => {
            cancelled = true
        }
    }, [onListSessions])

    return (
        <div style={{ display: 'flex', height: '100%' }}>
            <div
                data-testid="decision-session-list"
                style={{ width: 220, borderRight: '1px solid #1e2333', overflowY: 'auto', flexShrink: 0 }}
            >
                {sessions.length === 0 ? (
                    <div
                        data-testid="decision-session-empty"
                        style={{ padding: 12, color: '#374151', fontSize: 11, fontFamily: 'monospace', lineHeight: 1.6 }}
                    >
                        記録された意思決定セッションはまだありません
                    </div>
                ) : (
                    sessions.map((s) => (
                        <div
                            key={s.id}
                            data-testid={`decision-session-item-${s.id}`}
                            onClick={() => setSelectedId(s.id)}
                            style={{
                                padding: '8px 10px',
                                cursor: 'pointer',
                                borderBottom: '1px solid #1e2333',
                                background: selectedId === s.id ? '#1e3a5f' : 'transparent',
                            }}
                        >
                            <div style={{ fontSize: 11, color: '#e8eaf0', fontFamily: 'monospace' }}>{s.question}</div>
                            <div style={{ fontSize: 9, color: '#6b7280', marginTop: 2, fontFamily: 'monospace' }}>
                                {s.status} · {describeActionTaken(s.action_taken)}
                            </div>
                        </div>
                    ))
                )}
            </div>

            <div style={{ flex: 1, minWidth: 0 }}>
                {selectedId ? (
                    <DecisionGraphView
                        key={selectedId}
                        rootSessionId={selectedId}
                        onFetchSession={onFetchSession}
                        onFetchDecision={onFetchDecision}
                        onFetchConsidered={onFetchConsidered}
                    />
                ) : (
                    <div
                        data-testid="decision-graph-placeholder"
                        style={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            height: '100%',
                            color: '#374151',
                            fontSize: 12,
                            fontFamily: 'monospace',
                        }}
                    >
                        セッションを選択してください
                    </div>
                )}
            </div>
        </div>
    )
}
