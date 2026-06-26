import type { NodeProps } from '@xyflow/react'

export type CompanyNodeData = {
    code: string
    name: string
}

/**
 * ReactFlow カスタムノード: company
 * 証券コードと銘柄名を表示する
 */
export function CompanyNode({ data }: NodeProps & { data: CompanyNodeData }) {
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