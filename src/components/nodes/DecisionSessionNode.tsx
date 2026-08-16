import type { NodeProps } from '@xyflow/react'

export type DecisionSessionNodeData = {
    label: string
    statusLabel: string
    isExpanded: boolean
    isSelected: boolean
    onClick: () => void
}

/**
 * ReactFlow カスタムノード: decision_session
 * question を表示し、展開状態をアイコンで示す。クリックで展開/折りたたみと同時に選択もする。
 */
export function DecisionSessionNode({ data }: NodeProps & { data: DecisionSessionNodeData }) {
    return (
        <div
            data-testid="node-decision-session"
            onClick={data.onClick}
            style={{
                background: '#171b26',
                border: `1px solid ${data.isSelected ? '#e8eaf0' : '#3b82f6'}`,
                boxShadow: data.isSelected ? '0 0 0 2px rgba(255, 255, 255, 0.25)' : undefined,
                borderRadius: 6,
                padding: '8px 14px',
                fontFamily: 'monospace',
                minWidth: 180,
                textAlign: 'center',
                cursor: 'pointer',
            }}
        >
            <div style={{ fontSize: 8, fontWeight: 700, letterSpacing: '0.1em', textTransform: 'uppercase', color: '#3b82f6', marginBottom: 3 }}>
                Decision Session
            </div>
            <div style={{ fontSize: 10.5, color: '#e8eaf0', fontWeight: 600, lineHeight: 1.4 }}>{data.label}</div>
            <div style={{ fontSize: 8, color: '#6b7280', marginTop: 4 }}>
                {data.isExpanded ? '▼ 展開中' : '▶ 折りたたみ'} · {data.statusLabel}
            </div>
        </div>
    )
}
