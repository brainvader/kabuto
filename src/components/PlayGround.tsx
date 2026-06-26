import { useEffect, useMemo } from 'react'
import { useState } from 'react'
import {
    ReactFlow,
    Background,
    BackgroundVariant,
    type Node,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import { useKabutoStore } from '@/store/useKabutoStore'

type Tab = 'GRAPH' | 'RESULT'

// ── カスタムノード: company ───────────────────────────────────────────────────

function CompanyNode({ data }: { data: { code: string; name: string } }) {
    return (
        <div
            style={{
                background: '#1e3a5f',
                border: '2px solid #3b82f6',
                borderRadius: 8,
                padding: '16px 24px',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: 6,
                minWidth: 120,
            }}
        >
            <span style={{ fontFamily: 'monospace', fontSize: 9, fontWeight: 700, letterSpacing: '0.1em', color: '#3b82f6', textTransform: 'uppercase' }}>
                company
            </span>
            <span style={{ fontFamily: 'monospace', fontSize: 18, fontWeight: 700, color: '#e8eaf0' }}>
                {data.code}
            </span>
            <span style={{ fontSize: 11, color: '#6b7280' }}>
                {data.name}
            </span>
        </div>
    )
}

const nodeTypes = { company: CompanyNode }

// ── PlayGround ────────────────────────────────────────────────────────────────

/**
 * PlayGround パネル
 * - GRAPH タブ: 選択銘柄ノードを @xyflow/react キャンバスに表示
 * - RESULT タブ: 分析結果の表示（後続実装）
 */
export function PlayGround() {
    const [activeTab, setActiveTab] = useState<Tab>('GRAPH')
    const selection = useKabutoStore((s) => s.selection)

    const nodes: Node[] = useMemo(() => {
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
                {(['GRAPH', 'RESULT'] as Tab[]).map((tab) => (
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
                            style={{ position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#374151', fontSize: 12, fontFamily: 'monospace' }}
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
                    style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#374151', fontSize: 12, fontFamily: 'monospace' }}
                >
                    RESULT AREA
                </div>
            )}
        </div>
    )
}