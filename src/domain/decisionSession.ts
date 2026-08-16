import type { ConsideredEdge } from '@/lib/decisions'

/** "decision:xxx" | "decision_session:xxx" 形式の参照文字列を table/id に分解する */
export function parseTarget(target: string): { table: 'decision' | 'decision_session'; id: string } {
    const sep = target.indexOf(':')
    const table = target.slice(0, sep)
    const id = target.slice(sep + 1)
    if (table !== 'decision' && table !== 'decision_session') {
        throw new Error(`unknown target table: ${table}`)
    }
    return { table, id }
}

/** selection_mode の表示ラベル */
export function describeSelectionMode(mode: string): string {
    return mode === 'composite' ? '統合（AND）' : '排他選択'
}

/** action_taken の表示ラベル */
export function describeActionTaken(action: string): string {
    switch (action) {
        case 'bought':
            return '買い'
        case 'sold':
            return '売り'
        case 'watched':
            return '様子見'
        default:
            return '未確定'
    }
}

/** グラフ上の1ノード（decision または decision_session）。展開済みの子のみを持つ */
export interface DecisionNodeInfo {
    id: string
    table: 'decision' | 'decision_session'
    children: DecisionNodeInfo[]
}

/**
 * ルートセッションから、展開済み（expanded）のセッションだけをたどって
 * 現在画面に見えているべきツリー構造を組み立てる。
 * 未展開のセッション・未取得のエッジは子を持たない葉として扱う。
 */
export function buildVisibleTree(
    rootId: string,
    rootTable: 'decision' | 'decision_session',
    edgesBySession: Record<string, ConsideredEdge[]>,
    expanded: Set<string>,
): DecisionNodeInfo {
    const children: DecisionNodeInfo[] = []
    if (rootTable === 'decision_session' && expanded.has(rootId)) {
        const edges = edgesBySession[rootId] ?? []
        for (const edge of edges) {
            const { table, id } = parseTarget(edge.target)
            children.push(buildVisibleTree(id, table, edgesBySession, expanded))
        }
    }
    return { id: rootId, table: rootTable, children }
}

export interface LayoutPosition {
    x: number
    y: number
}

const X_GAP = 240
const Y_GAP = 150

/**
 * 単純な木構造レイアウト: 葉ノードを左から順に並べ、親ノードは子の中央に配置する。
 * dagre 等の依存を増やさずに済む最小限のアルゴリズム。
 */
export function layoutTree(root: DecisionNodeInfo): Record<string, LayoutPosition> {
    const positions: Record<string, LayoutPosition> = {}
    let nextLeafSlot = 0

    function visit(node: DecisionNodeInfo, depth: number): number {
        if (node.children.length === 0) {
            const x = nextLeafSlot * X_GAP
            nextLeafSlot += 1
            positions[node.id] = { x, y: depth * Y_GAP }
            return x
        }
        const childXs = node.children.map((child) => visit(child, depth + 1))
        const x = (childXs[0] + childXs[childXs.length - 1]) / 2
        positions[node.id] = { x, y: depth * Y_GAP }
        return x
    }

    visit(root, 0)
    return positions
}
