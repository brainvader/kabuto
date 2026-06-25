import { describe, it, expect, beforeEach } from 'vitest'
import { act, renderHook } from '@testing-library/react'

import { useKabutoStore } from '@/store/useKabutoStore'

// ── 型（実装ファイルが export する） ────────────────────────────────────────
import type { Selection, PipelineStatus } from '@/store/useKabutoStore'

// ── 初期状態リセット ─────────────────────────────────────────────────────────
beforeEach(() => {
    useKabutoStore.setState({
        selection: null,
        pipeline: { status: 'idle', activeId: null },
    })
})

// ── selection ───────────────────────────────────────────────────────────────

describe('selection', () => {
    it('初期値は null である', () => {
        const { result } = renderHook(() => useKabutoStore())
        expect(result.current.selection).toBeNull()
    })

    it('setSelection で銘柄を選択できる', () => {
        const { result } = renderHook(() => useKabutoStore())
        const stock: Selection = { code: '1605', name: 'INPEX' }

        act(() => result.current.setSelection(stock))

        expect(result.current.selection).toEqual(stock)
    })

    it('setSelection を再度呼ぶと上書きされる', () => {
        const { result } = renderHook(() => useKabutoStore())

        act(() => result.current.setSelection({ code: '1605', name: 'INPEX' }))
        act(() => result.current.setSelection({ code: '8306', name: '三菱UFJ' }))

        expect(result.current.selection).toEqual({ code: '8306', name: '三菱UFJ' })
    })

    it('clearSelection で null に戻る', () => {
        const { result } = renderHook(() => useKabutoStore())

        act(() => result.current.setSelection({ code: '1605', name: 'INPEX' }))
        act(() => result.current.clearSelection())

        expect(result.current.selection).toBeNull()
    })
})

// ── pipeline ────────────────────────────────────────────────────────────────

describe('pipeline', () => {
    it('初期値は idle / activeId null である', () => {
        const { result } = renderHook(() => useKabutoStore())
        expect(result.current.pipeline).toEqual({ status: 'idle', activeId: null })
    })

    it('setPipelineStatus("running", id) でパイプラインを開始できる', () => {
        const { result } = renderHook(() => useKabutoStore())

        act(() => result.current.setPipelineStatus('running', 'oil_sensitivity'))

        expect(result.current.pipeline).toEqual({
            status: 'running',
            activeId: 'oil_sensitivity',
        })
    })

    it('setPipelineStatus("idle") で停止し activeId が null になる', () => {
        const { result } = renderHook(() => useKabutoStore())

        act(() => result.current.setPipelineStatus('running', 'oil_sensitivity'))
        act(() => result.current.setPipelineStatus('idle'))

        expect(result.current.pipeline).toEqual({ status: 'idle', activeId: null })
    })
})

// ── 複数フック間での共有 ───────────────────────────────────────────────────

describe('store の共有', () => {
    it('別々の renderHook でも同じ状態を参照する', () => {
        const a = renderHook(() => useKabutoStore())
        const b = renderHook(() => useKabutoStore())

        act(() => a.result.current.setSelection({ code: '7203', name: 'トヨタ' }))

        expect(b.result.current.selection).toEqual({ code: '7203', name: 'トヨタ' })
    })
})