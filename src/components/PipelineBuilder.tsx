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
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            {/* タブバー */}
            <div role="tablist" style={{ display: 'flex', gap: 2 }}>
                {(['BUILDER', 'RESULT'] as Tab[]).map((tab) => (
                    <button
                        key={tab}
                        role="tab"
                        aria-selected={activeTab === tab}
                        onClick={() => setActiveTab(tab)}
                    >
                        {tab}
                    </button>
                ))}
            </div>

            {/* コンテンツ */}
            {activeTab === 'BUILDER' && (
                <div data-testid="pipeline-canvas" style={{ flex: 1 }} />
            )}
            {activeTab === 'RESULT' && (
                <div data-testid="pipeline-result" style={{ flex: 1 }} />
            )}
        </div>
    )
}