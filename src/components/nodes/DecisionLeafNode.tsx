import type { NodeProps } from '@xyflow/react'

export type DecisionLeafNodeData = {
    label: string
    correlation: number | null
    confirmed: boolean
    isSelected: boolean
    rejectionReason: string | null
    onClick: () => void
}

/**
 * ReactFlow カスタムノード: decision
 * hypothesis と相関係数を表示し、CONSIDEREDエッジの採用/却下を色で示す。
 */
export function DecisionLeafNode({ data }: NodeProps & { data: DecisionLeafNodeData }) {
    const accepted = data.isSelected
    return (
        <div
            data-testid="node-decision"
            onClick={data.onClick}
            style={{
                background: accepted ? '#064e35' : '#171b26',
                border: `1px solid ${accepted ? '#10b981' : '#1e2333'}`,
                opacity: accepted ? 1 : 0.6,
                borderRadius: 4,
                padding: '6px 10px',
                fontFamily: 'monospace',
                minWidth: 150,
                textAlign: 'center',
                cursor: 'pointer',
            }}
        >
            <div style={{ fontSize: 7, color: '#6b7280', textTransform: 'uppercase', marginBottom: 2 }}>Decision</div>
            <div style={{ fontSize: 9.5, fontWeight: 600, color: '#e8eaf0' }}>{data.label}</div>
            {data.correlation !== null && (
                <div style={{ fontFamily: 'monospace', fontSize: 11, fontWeight: 700, marginTop: 3, color: data.confirmed ? '#10b981' : '#ef4444' }}>
                    {data.correlation.toFixed(2)}
                </div>
            )}
            <div
                style={{
                    display: 'inline-block',
                    marginTop: 4,
                    fontSize: 7,
                    fontWeight: 700,
                    padding: '1px 6px',
                    borderRadius: 999,
                    background: accepted ? '#064e35' : '#450a0a',
                    color: accepted ? '#10b981' : '#ef4444',
                }}
            >
                {accepted ? '✓ 採用' : '✗ 却下'}
            </div>
            {!accepted && data.rejectionReason && (
                <div style={{ fontSize: 7.5, color: '#ef4444', fontStyle: 'italic', marginTop: 3 }}>{data.rejectionReason}</div>
            )}
        </div>
    )
}
