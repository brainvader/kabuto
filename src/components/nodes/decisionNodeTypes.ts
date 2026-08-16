import { DecisionSessionNode } from './DecisionSessionNode'
import { DecisionLeafNode } from './DecisionLeafNode'

/**
 * DecisionGraphView 用の ReactFlow カスタムノード一覧。
 * コンポーネント内で定義すると再レンダー毎にリセットされるため、モジュールスコープに置く。
 */
export const decisionNodeTypes = {
    decisionSession: DecisionSessionNode,
    decisionLeaf: DecisionLeafNode,
} as const
