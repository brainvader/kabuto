import { useState } from 'react'

type Tab = 'GRAPH' | 'RESULT'

/**
 * PlayGround パネル
 * - GRAPH タブ: 選択銘柄ノードとその関連ノードを表示
 * - RESULT タブ: 分析結果の表示（後続実装）
 */
export function PlayGround() {
    const [activeTab, setActiveTab] = useState<Tab>('GRAPH')

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

            {/* コンテンツ */}
            {activeTab === 'GRAPH' && (
                <div
                    data-testid="playground-canvas"
                    style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#374151', fontSize: 12, fontFamily: 'monospace' }}
                >
                    GRAPH AREA
                </div>
            )}
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