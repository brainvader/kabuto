/**
 * @file docs/bom/ctx-aip.ts
 * @description ctx-aip — Gemini Function Calling / CoT Debugger / Approval Gate の型定義
 *
 * 責務:
 * - global-store.selection を受け取り Gemini へ送信
 * - Gemini の CoT(text) + functionCall を表示
 * - operationPolicy に従い承認レベルを自動判定
 * - 承認後に Tauri Bridge 経由で execute_script を呼び出す
 * - 結果を reports/ に JSON ファイルとして出力
 *
 * DB アクセス: なし（FalkorDB への書き戻し禁止）
 */

import { z } from 'zod';

// ─── Gemini Response Parts ────────────────────────────────────────────────────

export const GeminiFunctionCallSchema = z.object({
    name: z.string(),
    args: z.record(z.string(), z.unknown()),
});

export type GeminiFunctionCall = z.infer<typeof GeminiFunctionCallSchema>;

export const GeminiResponseSchema = z.object({
    thought: z.string(),
    functionCall: GeminiFunctionCallSchema.optional(),
    /** run_pipeline 経由の場合に使用 */
    pipelineId: z.string().optional(),
});

export type GeminiResponse = z.infer<typeof GeminiResponseSchema>;

// ─── Approval Gate ────────────────────────────────────────────────────────────

export const ApprovalStateSchema = z.enum([
    'idle',
    'awaiting_approval',
    'awaiting_confirmation',
    'executing',
    'done',
    'cancelled',
    'error',
]);

export type ApprovalState = z.infer<typeof ApprovalStateSchema>;

export const ApprovalRequestSchema = z.object({
    state: ApprovalStateSchema,
    policy: z.enum(['simulation', 'fact_write']),
    proposed: GeminiResponseSchema,
});

export type ApprovalRequest = z.infer<typeof ApprovalRequestSchema>;

// ─── Simulation Report ────────────────────────────────────────────────────────

export const SimulationReportSchema = z.object({
    id: z.string(),
    timestamp: z.string(),
    action: z.string(),
    selection_id: z.string(),
    result: z.record(z.string(), z.unknown()),
    /** レポートファイルの出力先パス */
    report_path: z.string(),
    narrative: z.string().optional(),
});

export type SimulationReport = z.infer<typeof SimulationReportSchema>;

// ─── Tauri Bridge ─────────────────────────────────────────────────────────────

/** execute_script Tauri コマンドの引数型 */
export interface ExecuteScriptArgs {
    script: string;
    args: Record<string, unknown>;
}

/** execute_script Tauri コマンドの戻り値型 */
export interface ExecuteScriptResult {
    success: boolean;
    report_path: string;
    error?: string;
}

// ─── ctx-aip Props ────────────────────────────────────────────────────────────

export interface AipPanelProps {
    /** 承認後のレポートパスを ctx-metrics に通知するコールバック */
    onReportReady: (reportPath: string) => void;
    /** CoT 生成後に ctx-graph を COT モードに切り替えるコールバック */
    onCotGenerated: () => void;
}