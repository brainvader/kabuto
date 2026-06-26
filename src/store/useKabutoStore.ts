import { create } from 'zustand'

// ── 型 ──────────────────────────────────────────────────────────────────────

export type Selection = {
    /** 証券コード（例: "1605"） */
    code: string
    /** 銘柄名（例: "INPEX"） */
    name: string
}

export type PipelineStatus = 'idle' | 'running'

type PipelineState = {
    status: PipelineStatus
    activeId: string | null
}

// ── Store ────────────────────────────────────────────────────────────────────

type KabutoStore = {
    // State
    selection: Selection | null
    pipeline: PipelineState

    // Actions
    setSelection: (selection: Selection) => void
    clearSelection: () => void
    setPipelineStatus: (status: PipelineStatus, activeId?: string | null) => void
}

export const useKabutoStore = create<KabutoStore>()((set) => ({
    selection: null,
    pipeline: { status: 'idle', activeId: null },

    setSelection: (selection) => set({ selection }),

    clearSelection: () => set({ selection: null }),

    setPipelineStatus: (status, activeId = null) =>
        set({ pipeline: { status, activeId: status === 'idle' ? null : activeId } }),
}))