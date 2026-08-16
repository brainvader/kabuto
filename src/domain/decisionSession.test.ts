import { describe, it, expect } from 'vitest'

import {
    parseTarget,
    describeSelectionMode,
    describeActionTaken,
    buildVisibleTree,
    layoutTree,
    type DecisionNodeInfo,
} from './decisionSession'
import type { ConsideredEdge } from '@/lib/decisions'

describe('parseTarget', () => {
    it('"decision:xxx" を decision テーブルのidに分解する', () => {
        expect(parseTarget('decision:INPEX_WTI_2026Q3')).toEqual({ table: 'decision', id: 'INPEX_WTI_2026Q3' })
    })

    it('"decision_session:xxx" を decision_session テーブルのidに分解する', () => {
        expect(parseTarget('decision_session:macro')).toEqual({ table: 'decision_session', id: 'macro' })
    })

    it('未知のテーブル名は例外を投げる', () => {
        expect(() => parseTarget('macro:WTI')).toThrow()
    })
})

describe('describeSelectionMode', () => {
    it('composite は「統合（AND）」と表示する', () => {
        expect(describeSelectionMode('composite')).toBe('統合（AND）')
    })

    it('exclusive は「排他選択」と表示する', () => {
        expect(describeSelectionMode('exclusive')).toBe('排他選択')
    })
})

describe('describeActionTaken', () => {
    it.each([
        ['bought', '買い'],
        ['sold', '売り'],
        ['watched', '様子見'],
        ['none', '未確定'],
    ])('%s は %s と表示する', (action, expected) => {
        expect(describeActionTaken(action)).toBe(expected)
    })
})

describe('buildVisibleTree', () => {
    it('展開していないセッションは子を持たない', () => {
        const tree = buildVisibleTree('root', 'decision_session', {}, new Set())
        expect(tree).toEqual({ id: 'root', table: 'decision_session', children: [] })
    })

    it('展開済みのセッションはCONSIDEREDエッジをたどって子を持つ', () => {
        const edges: Record<string, ConsideredEdge[]> = {
            root: [
                { target: 'decision_session:macro', selected: true, rejection_reason: null },
                { target: 'decision:risk_check', selected: true, rejection_reason: null },
            ],
        }
        const tree = buildVisibleTree('root', 'decision_session', edges, new Set(['root']))
        expect(tree.children).toEqual([
            { id: 'macro', table: 'decision_session', children: [] },
            { id: 'risk_check', table: 'decision', children: [] },
        ])
    })

    it('入れ子のセッションも展開済みなら再帰的にたどる', () => {
        const edges: Record<string, ConsideredEdge[]> = {
            root: [{ target: 'decision_session:macro', selected: true, rejection_reason: null }],
            macro: [{ target: 'decision:wti', selected: true, rejection_reason: null }],
        }
        const tree = buildVisibleTree('root', 'decision_session', edges, new Set(['root', 'macro']))
        expect(tree.children[0].children).toEqual([{ id: 'wti', table: 'decision', children: [] }])
    })

    it('decision ノードは展開対象にならない（葉のまま）', () => {
        const edges: Record<string, ConsideredEdge[]> = {
            leaf: [{ target: 'decision_session:should_not_appear', selected: true, rejection_reason: null }],
        }
        const tree = buildVisibleTree('leaf', 'decision', edges, new Set(['leaf']))
        expect(tree.children).toEqual([])
    })
})

describe('layoutTree', () => {
    it('葉ノード1つのツリーは原点に配置される', () => {
        const tree: DecisionNodeInfo = { id: 'a', table: 'decision_session', children: [] }
        expect(layoutTree(tree)).toEqual({ a: { x: 0, y: 0 } })
    })

    it('複数の葉は横に等間隔で並び、親は子の中央に配置される', () => {
        const tree: DecisionNodeInfo = {
            id: 'root',
            table: 'decision_session',
            children: [
                { id: 'left', table: 'decision', children: [] },
                { id: 'right', table: 'decision', children: [] },
            ],
        }
        const positions = layoutTree(tree)
        expect(positions.left.x).toBeLessThan(positions.right.x)
        expect(positions.root.x).toBeCloseTo((positions.left.x + positions.right.x) / 2)
        expect(positions.root.y).toBe(0)
        expect(positions.left.y).toBeGreaterThan(positions.root.y)
        expect(positions.left.y).toBe(positions.right.y)
    })
})
