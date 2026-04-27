/**
 * @file docs/bom/ctx-graph.ts
 * @description ctx-graph — 因果グラフ / CoT フロー / パイプラインビルダー の型定義
 *
 * 責務:
 * - FalkorDB の可視化（Read only）
 * - execution.mode に応じた 3 モード切替 (GRAPH / COT / PIPELINE)
 * - node_click → global-store.selection 更新
 * - pipeline_saved → pipelineRegistry 登録
 */

import { z } from 'zod';
import type { Node, Edge } from '@xyflow/react';

// ─── Graph Node Types ─────────────────────────────────────────────────────────

export const GraphNodeTypeSchema = z.enum(['company', 'sector', 'index', 'macro_factor']);
export type GraphNodeType = z.infer<typeof GraphNodeTypeSchema>;

export const GraphNodeDataSchema = z.object({
    id: z.string(),
    label: z.string(),
    type: GraphNodeTypeSchema,
    /** Polars で計算したリアルタイム数値（ズーム拡大時に表示） */
    metrics: z
        .object({
            price: z.number().optional(),
            change_pct: z.number().optional(),
            market_cap: z.number().optional(),
        })
        .optional(),
    selected: z.boolean().default(false),
});

export type GraphNodeData = z.infer<typeof GraphNodeDataSchema>;

export type KabutoNode = Node<GraphNodeData>;

// ─── Graph Edge Types ─────────────────────────────────────────────────────────

export const GraphEdgeDataSchema = z.object({
    /** 因果関係の強さ（0〜1）: エッジの太さ・透過度に反映 */
    weight: z.number().min(0).max(1),
    label: z.string().optional(),
});

export type GraphEdgeData = z.infer<typeof GraphEdgeDataSchema>;

export type KabutoEdge = Edge<GraphEdgeData>;

// ─── CoT Node (Gemini 実行計画) ───────────────────────────────────────────────

export const CoTNodeTypeSchema = z.enum(['thought', 'function_call', 'result']);
export type CoTNodeType = z.infer<typeof CoTNodeTypeSchema>;

export const CoTNodeDataSchema = z.object({
    type: CoTNodeTypeSchema,
    label: z.string(),
    detail: z.string().optional(),
});

export type CoTNodeData = z.infer<typeof CoTNodeDataSchema>;

// ─── Pipeline Builder ─────────────────────────────────────────────────────────

export const PipelineNodeDataSchema = z.object({
    actionId: z.string(),
    label: z.string(),
    args: z.record(z.string(), z.unknown()),
    configured: z.boolean().default(false),
});

export type PipelineNodeData = z.infer<typeof PipelineNodeDataSchema>;

// ─── Graph Mode Props ─────────────────────────────────────────────────────────

export interface GraphModeProps {
    nodes: KabutoNode[];
    edges: KabutoEdge[];
    onNodeClick: (nodeId: string) => void;
}

export interface CoTModeProps {
    nodes: Node<CoTNodeData>[];
    edges: Edge[];
}

export interface PipelineModeProps {
    onSave: (id: string, label: string, description: string) => void;
}