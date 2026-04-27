/**
 * @file docs/bom/global-store.ts
 * @description Kabuto Global Store — Zustand で管理するアプリ全体の共有状態
 *
 * 設計方針:
 * - FalkorDB は事実の保管庫。シミュレーションから書き戻しは行わない。
 * - execution.mode が ctx-graph のレンダリングモードを制御する。
 * - operationPolicy が各操作の承認レベルを決定する。
 */

import { z } from 'zod';

// ─── Selection ───────────────────────────────────────────────────────────────

export const SelectionSchema = z.object({
    id: z.string(),
    type: z.enum(['Company', 'Sector', 'Index']),
    name: z.string(),
});

export type Selection = z.infer<typeof SelectionSchema>;

// ─── Execution ───────────────────────────────────────────────────────────────

export const ExecutionModeSchema = z.enum(['GRAPH', 'COT', 'PIPELINE']);
export type ExecutionMode = z.infer<typeof ExecutionModeSchema>;

export const ExecutionStatusSchema = z.enum(['idle', 'pending', 'executing', 'done', 'error']);
export type ExecutionStatus = z.infer<typeof ExecutionStatusSchema>;

export const ExecutionSchema = z.object({
    status: ExecutionStatusSchema,
    mode: ExecutionModeSchema,
});

export type Execution = z.infer<typeof ExecutionSchema>;

// ─── OperationPolicy ─────────────────────────────────────────────────────────

export const OperationPolicySchema = z.object({
    read: z.literal('auto'),
    simulation: z.literal('requires_approval'),
    fact_write: z.literal('requires_confirmation'),
});

export type OperationPolicy = z.infer<typeof OperationPolicySchema>;

// ─── ActionRegistry ──────────────────────────────────────────────────────────

export const ActionPolicySchema = z.enum(['read', 'simulation', 'fact_write']);
export type ActionPolicy = z.infer<typeof ActionPolicySchema>;

export const ActionDefinitionSchema = z.object({
    when: z.string(),
    input: z.array(z.string()),
    output: z.array(z.string()),
    executor: z.enum(['execute_script', 'pipeline_runner']),
    script: z.string().optional(),
    writes_to: z.enum(['simulation', 'falkordb']),
    policy: ActionPolicySchema,
});

export type ActionDefinition = z.infer<typeof ActionDefinitionSchema>;

export const ActionRegistrySchema = z.record(z.string(), ActionDefinitionSchema);
export type ActionRegistry = z.infer<typeof ActionRegistrySchema>;

// ─── PipelineRegistry ────────────────────────────────────────────────────────

export const PipelineStepSchema = z.object({
    action: z.string(),
    args: z.record(z.string(), z.unknown()),
});

export type PipelineStep = z.infer<typeof PipelineStepSchema>;

export const PipelineDefinitionSchema = z.object({
    label: z.string(),
    description: z.string(),
    steps: z.array(PipelineStepSchema),
    input: z.array(z.string()),
    output: z.string(),
});

export type PipelineDefinition = z.infer<typeof PipelineDefinitionSchema>;

export const PipelineRegistrySchema = z.record(z.string(), PipelineDefinitionSchema);
export type PipelineRegistry = z.infer<typeof PipelineRegistrySchema>;

// ─── UIDataFlow ──────────────────────────────────────────────────────────────

export const UIDataFlowSchema = z.object({
    from: z.string(),
    to: z.string(),
    data: z.string(),
    trigger: z.string(),
});

export type UIDataFlow = z.infer<typeof UIDataFlowSchema>;

// ─── GlobalStore (Zustand state shape) ───────────────────────────────────────

export const GlobalStoreSchema = z.object({
    project: z.literal('Kabuto'),
    selection: SelectionSchema,
    execution: ExecutionSchema,
    operationPolicy: OperationPolicySchema,
    actionRegistry: ActionRegistrySchema,
    pipelineRegistry: PipelineRegistrySchema,
    uiDataFlows: z.array(UIDataFlowSchema),
});

export type GlobalStore = z.infer<typeof GlobalStoreSchema>;

// ─── Zustand Actions (型定義のみ) ─────────────────────────────────────────────

export interface GlobalStoreActions {
    setSelection: (selection: Selection) => void;
    setExecutionMode: (mode: ExecutionMode) => void;
    setExecutionStatus: (status: ExecutionStatus) => void;
    registerPipeline: (id: string, def: PipelineDefinition) => void;
}