import { useMemo, useState } from 'react'
import {
    ReactFlow,
    Background,
    BackgroundVariant,
    type Node,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useKabutoStore } from '@/store/useKabutoStore'
import { nodeTypes } from '@/components/nodes/nodeTypes'
import type { CompanyNodeData } from '@/components/nodes/CompanyNode'
import { DecisionSessionsPanel } from '@/components/DecisionSessionsPanel'

type Tab = 'GRAPH' | 'RESULT' | 'DECISION'

/**
 * PlayGround パネル
 * - GRAPH タブ: 選択銘柄ノードを @xyflow/react キャンバスに表示
 * - RESULT タブ: 分析結果の表示（後続実装）
 * - DECISION タブ: 記録済みの decision_session を一覧・グラフ表示（007.md Step 7）
 */
export function PlayGround() {
    const [activeTab, setActiveTab] = useState<Tab>('GRAPH')
    const selection = useKabutoStore((s) => s.selection)

    const nodes: Node<CompanyNodeData>[] = useMemo(() => {
        if (!selection) return []
        return [
            {
                id: `company-${selection.code}`,
                type: 'company',
                position: { x: 0, y: 0 },
                data: { code: selection.code, name: selection.name },
            },
        ]
    }, [selection])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            {/* タブバー */}
            <div role="tablist" style={{ display: 'flex', gap: 2, padding: '6px 10px', background: '#13161f', borderBottom: '1px solid #1e2333', flexShrink: 0 }}>
                {(['GRAPH', 'RESULT', 'DECISION'] as Tab[]).map((tab) => (
                    <button
                        key={tab}
                        role="tab"
                        aria-selected={activeTab === tab}
                        onClick={() => setActiveTab(tab)}
                        style={{
                            padding: '3px 10px',
                            borderRadius: 2,
                            fontFamily: 'monospace',
                            fontSize: 9,
                            fontWeight: 700,
                            letterSpacing: '0.08em',
                            cursor: 'pointer',
                            border: '1px solid',
                            borderColor: activeTab === tab ? '#3b82f6' : '#1e2333',
                            background: activeTab === tab ? '#1e3a5f' : 'transparent',
                            color: activeTab === tab ? '#3b82f6' : '#6b7280',
                        }}
                    >
                        {tab}
                    </button>
                ))}
            </div>

            {/* GRAPH タブ */}
            {activeTab === 'GRAPH' && (
                <div data-testid="playground-canvas" style={{ flex: 1, position: 'relative' }}>
                    {selection === null ? (
                        <div
                            data-testid="playground-empty"
                            style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#6b7280', fontSize: 12, fontFamily: 'monospace' }}
                        >
                            銘柄を選択してください
                        </div>
                    ) : (
                        <div data-testid="node-company" style={{ width: '100%', height: '100%' }}>
                            <ReactFlow
                                nodes={nodes}
                                edges={[]}
                                nodeTypes={nodeTypes}
                                fitView
                                colorMode="dark"
                                proOptions={{ hideAttribution: true }}
                            >
                                <Background variant={BackgroundVariant.Dots} color="#1e2333" gap={28} size={0.5} />
                            </ReactFlow>
                        </div>
                    )}
                </div>
            )}

            {/* RESULT タブ */}
            {activeTab === 'RESULT' && (
                <div
                    data-testid="playground-result"
                    style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#6b7280', fontSize: 12, fontFamily: 'monospace' }}
                >
                    RESULT AREA
                </div>
            )}

            {/* DECISION タブ */}
            {activeTab === 'DECISION' && (
                <div data-testid="playground-decision" style={{ flex: 1, position: 'relative' }}>
                    <DecisionSessionsPanel />
                </div>
            )}
        </div>
    )
}