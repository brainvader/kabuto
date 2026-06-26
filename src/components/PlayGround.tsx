import { useState } from 'react'
import { useKabutoStore } from '@/store/useKabutoStore'

type Tab = 'GRAPH' | 'RESULT'

/**
 * PlayGround パネル
 * - GRAPH タブ: 選択銘柄ノードとその関連ノードを表示
 * - RESULT タブ: 分析結果の表示（後続実装）
 */
export function PlayGround() {
    const [activeTab, setActiveTab] = useState<Tab>('GRAPH')
    const selection = useKabutoStore((s) => s.selection)

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            {/* タブバー */}
            <div role="tablist" style={{ display: 'flex', gap: 2, padding: '6px 10px', background: '#13161f', borderBottom: '1px solid #1e2333' }}>
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
                <div
                    data-testid="playground-canvas"
                    style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'center', justifyContent: 'center', background: '#0f1117' }}
                >
                    {selection === null ? (
                        <div
                            data-testid="playground-empty"
                            style={{ color: '#374151', fontSize: 12, fontFamily: 'monospace' }}
                        >
                            銘柄を選択してください
                        </div>
                    ) : (
                        <div
                            data-testid="node-company"
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
                                {selection.code}
                            </span>
                            <span style={{ fontSize: 11, color: '#6b7280' }}>
                                {selection.name}
                            </span>
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