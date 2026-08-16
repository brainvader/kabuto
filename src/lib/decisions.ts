import { invoke } from '@tauri-apps/api/core'

/** 1つの仮説検証の結果（src-tauri/src/decision/types.rs の Decision と対応） */
export interface Decision {
    hypothesis: string
    method: string
    correlation: number | null
    confirmed: boolean
    lag_days: number | null
    based_on_doc_ids: string[]
}

/** 選択肢の集合と結論（同 DecisionSession と対応） */
export interface DecisionSession {
    question: string
    company: string | null
    status: string
    selection_mode: string
    conclusion: string | null
    action_taken: string
}

/** 思考の切っ掛け（同 Trigger と対応） */
export interface Trigger {
    type: string
    description: string
    source_ref: string | null
}

/** decision_session の CONSIDERED エッジ1本分。target は "decision:xxx" | "decision_session:xxx" */
export interface ConsideredEdge {
    target: string
    selected: boolean
    rejection_reason: string | null
}

export function fetchDecisionSession(id: string): Promise<DecisionSession | null> {
    return invoke('get_decision_session_cmd', { id })
}

export function fetchDecision(id: string): Promise<Decision | null> {
    return invoke('get_decision_cmd', { id })
}

export function fetchConsidered(sessionId: string): Promise<ConsideredEdge[]> {
    return invoke('list_considered_cmd', { sessionId })
}

export function listDecisionSessions(): Promise<DecisionSession[]> {
    return invoke('list_decision_sessions_cmd')
}

export function createTrigger(trigger: Trigger): Promise<string> {
    return invoke('create_trigger_cmd', { trigger })
}

export function createDecisionSession(session: DecisionSession): Promise<string> {
    return invoke('create_decision_session_cmd', { session })
}

export function createDecision(decision: Decision): Promise<string> {
    return invoke('create_decision_cmd', { decision })
}

export function relatePrompted(triggerId: string, sessionId: string): Promise<void> {
    return invoke('relate_prompted_cmd', { triggerId, sessionId })
}

export function relateConsidered(
    sessionId: string,
    targetTable: 'decision' | 'decision_session',
    targetId: string,
    selected: boolean,
    rejectionReason: string | null,
): Promise<void> {
    return invoke('relate_considered_cmd', {
        sessionId,
        targetTable,
        targetId,
        selected,
        rejectionReason,
    })
}

export function resolveDecisionSession(
    id: string,
    conclusion: string,
    actionTaken: string,
): Promise<void> {
    return invoke('resolve_decision_session_cmd', { id, conclusion, actionTaken })
}
