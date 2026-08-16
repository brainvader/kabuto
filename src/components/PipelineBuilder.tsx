import { useState } from 'react'

type Tab = 'BUILDER' | 'RESULT'

/**
 * パイプラインビルダーパネル
 * - BUILDER タブ: @xyflow/react キャンバス（後続実装）
 * - RESULT タブ: OUTPUT ノードの結果表示（後続実装）
 */
export function PipelineBuilder() {
    const [activeTab, setActiveTab] = useState<Tab>('BUILDER')

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%', background: '#0f1117', color: '#e8eaf0' }}>
            {/* タブバー */}
            <div
                role="tablist"
                style={{ display: 'flex', gap: 2, padding: '8px 14px', background: '#13161f', borderBottom: '1px solid #1e2333' }}
            >
                {(['BUILDER', 'RESULT'] as Tab[]).map((tab) => (
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
            {activeTab === 'BUILDER' && (
                <div
                    data-testid="pipeline-canvas"
                    style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#6b7280', fontSize: 12, fontFamily: 'monospace' }}
                >
                    CANVAS AREA
                </div>
            )}
            {activeTab === 'RESULT' && (
                <div
                    data-testid="pipeline-result"
                    style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#6b7280', fontSize: 12, fontFamily: 'monospace' }}
                >
                    RESULT AREA
                </div>
            )}
        </div>
    )
}