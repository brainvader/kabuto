import { useCallback, useEffect, useMemo, useState } from 'react'
import { ReactFlow, Background, BackgroundVariant, type Node, type Edge } from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import {
    fetchDecisionSession,
    fetchDecision,
    fetchConsidered,
    type DecisionSession,
    type Decision,
    type ConsideredEdge,
} from '@/lib/decisions'
import { buildVisibleTree, layoutTree, parseTarget, describeSelectionMode } from '@/domain/decisionSession'
import type { DecisionSessionNodeData } from '@/components/nodes/DecisionSessionNode'
import type { DecisionLeafNodeData } from '@/components/nodes/DecisionLeafNode'
import { decisionNodeTypes } from '@/components/nodes/decisionNodeTypes'

export interface DecisionGraphViewProps {
    /** 表示するグラフの起点となる decision_session のID */
    rootSessionId: string
    onFetchSession?: (id: string) => Promise<DecisionSession | null>
    onFetchDecision?: (id: string) => Promise<Decision | null>
    onFetchConsidered?: (sessionId: string) => Promise<ConsideredEdge[]>
}

type SelectedRecord =
    | { table: 'decision_session'; id: string; data: DecisionSession }
    | { table: 'decision'; id: string; data: Decision }

/**
 * decision_session を起点にした決定グラフの表示。
 * セッションノードをクリックすると CONSIDERED エッジをたどって子ノードを展開し、
 * どのノードをクリックしても下部のJSONパネルに生データを表示する。
 */
export function DecisionGraphView({
    rootSessionId,
    onFetchSession = fetchDecisionSession,
    onFetchDecision = fetchDecision,
    onFetchConsidered = fetchConsidered,
}: DecisionGraphViewProps) {
    const [sessions, setSessions] = useState<Record<string, DecisionSession>>({})
    const [decisions, setDecisions] = useState<Record<string, Decision>>({})
    const [edgesBySession, setEdgesBySession] = useState<Record<string, ConsideredEdge[]>>({})
    const [expanded, setExpanded] = useState<Set<string>>(new Set())
    const [selected, setSelected] = useState<SelectedRecord | null>(null)

    useEffect(() => {
        let cancelled = false
        onFetchSession(rootSessionId).then((session) => {
            if (!cancelled && session) {
                setSessions((prev) => ({ ...prev, [rootSessionId]: session }))
            }
        })
        return () => {
            cancelled = true
        }
    }, [rootSessionId, onFetchSession])

    const handleSelectDecision = useCallback(
        (id: string) => {
            const data = decisions[id]
            if (data) setSelected({ table: 'decision', id, data })
        },
        [decisions],
    )

    const handleToggleSession = useCallback(
        async (id: string) => {
            const data = sessions[id]
            if (data) setSelected({ table: 'decision_session', id, data })

            setExpanded((prev) => {
                if (prev.has(id)) {
                    const next = new Set(prev)
                    next.delete(id)
                    return next
                }
                return new Set(prev).add(id)
            })

            if (edgesBySession[id]) return

            const edges = await onFetchConsidered(id)
            setEdgesBySession((prev) => ({ ...prev, [id]: edges }))

            for (const edge of edges) {
                const { table, id: targetId } = parseTarget(edge.target)
                if (table === 'decision_session') {
                    const s = await onFetchSession(targetId)
                    if (s) setSessions((prev) => ({ ...prev, [targetId]: s }))
                } else {
                    const d = await onFetchDecision(targetId)
                    if (d) setDecisions((prev) => ({ ...prev, [targetId]: d }))
                }
            }
        },
        [sessions, edgesBySession, onFetchConsidered, onFetchSession, onFetchDecision],
    )

    const tree = useMemo(
        () => buildVisibleTree(rootSessionId, 'decision_session', edgesBySession, expanded),
        [rootSessionId, edgesBySession, expanded],
    )
    const positions = useMemo(() => layoutTree(tree), [tree])

    const { nodes, edges } = useMemo(() => {
        const nodes: Node[] = []
        const edges: Edge[] = []

        const walk = (nodeId: string, table: 'decision' | 'decision_session', children: typeof tree.children, parentId?: string, edgeMeta?: ConsideredEdge) => {
            const position = positions[nodeId] ?? { x: 0, y: 0 }

            if (table === 'decision_session') {
                const session = sessions[nodeId]
                const data: DecisionSessionNodeData = {
                    label: session?.question ?? nodeId,
                    statusLabel: session ? describeSelectionMode(session.selection_mode) : '',
                    isExpanded: expanded.has(nodeId),
                    isSelected: selected?.id === nodeId,
                    onClick: () => handleToggleSession(nodeId),
                }
                nodes.push({ id: nodeId, type: 'decisionSession', position, data })
            } else {
                const decision = decisions[nodeId]
                const data: DecisionLeafNodeData = {
                    label: decision?.hypothesis ?? nodeId,
                    correlation: decision?.correlation ?? null,
                    confirmed: decision?.confirmed ?? false,
                    isSelected: edgeMeta?.selected ?? true,
                    rejectionReason: edgeMeta?.rejection_reason ?? null,
                    onClick: () => handleSelectDecision(nodeId),
                }
                nodes.push({ id: nodeId, type: 'decisionLeaf', position, data })
            }

            if (parentId) {
                edges.push({
                    id: `${parentId}->${nodeId}`,
                    source: parentId,
                    target: nodeId,
                    style: { stroke: edgeMeta?.selected === false ? '#4b5563' : '#10b981' },
                })
            }

            const childEdges = edgesBySession[nodeId] ?? []
            children.forEach((child, i) => {
                walk(child.id, child.table, child.children, nodeId, childEdges[i])
            })
        }

        walk(tree.id, tree.table, tree.children)
        return { nodes, edges }
    }, [tree, positions, sessions, decisions, edgesBySession, expanded, selected, handleToggleSession, handleSelectDecision])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div data-testid="decision-graph-canvas" style={{ flex: 1, position: 'relative', minHeight: 320 }}>
                <ReactFlow
                    nodes={nodes}
                    edges={edges}
                    nodeTypes={decisionNodeTypes}
                    fitView
                    colorMode="dark"
                    proOptions={{ hideAttribution: true }}
                >
                    <Background variant={BackgroundVariant.Dots} color="#1e2333" gap={28} size={0.5} />
                </ReactFlow>
            </div>

            <div
                data-testid="decision-json-panel"
                style={{
                    borderTop: '1px solid #1e2333',
                    background: '#13161f',
                    padding: '10px 14px',
                    maxHeight: 200,
                    overflow: 'auto',
                    flexShrink: 0,
                }}
            >
                <div style={{ fontFamily: 'monospace', fontSize: 9, letterSpacing: '0.08em', color: '#6b7280', marginBottom: 6 }}>
                    {selected ? `${selected.table}:${selected.id}` : 'ノードをクリックしてください'}
                </div>
                <pre
                    data-testid="decision-json-output"
                    style={{ fontFamily: 'monospace', fontSize: 11, color: '#e8eaf0', margin: 0, whiteSpace: 'pre-wrap' }}
                >
                    {selected ? JSON.stringify(selected.data, null, 2) : ''}
                </pre>
            </div>
        </div>
    )
}
