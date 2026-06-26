import { CompanyNode } from './CompanyNode'

/**
 * ReactFlow に登録するカスタムノード一覧
 * 新しいノード種別はここに追加する
 */
export const nodeTypes = {
    company: CompanyNode,
} as const