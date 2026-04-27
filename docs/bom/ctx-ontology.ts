/**
 * @file docs/bom/ctx-ontology.ts
 * @description ctx-ontology — JPX上場企業データをFalkorDBへ投入・スキーマ管理
 *
 * 責務:
 * - JPXマスターデータ (Code, Name, Sector, Market) の取得・キャッシュ
 * - FalkorDB へのスキーマ定義（因果グラフ用ノード・エッジテンプレート）
 * - fact_write 権限による確定データの書き込み
 * - operationPolicy: fact_write → requires_confirmation（UIで確認必須）
 */

import { z } from 'zod';

// ─── JPX Master Data ──────────────────────────────────────────────────────────

export const MarketSchema = z.enum(['PrimeMarket', 'StandardMarket', 'GrowthMarket']);
export type Market = z.infer<typeof MarketSchema>;

export const SectorSchema = z.enum([
    'Energy',
    'Materials',
    'Industrials',
    'ConsumerDiscretionary',
    'ConsumerStaples',
    'Healthcare',
    'Financials',
    'InformationTechnology',
    'CommunicationServices',
    'Utilities',
    'RealEstate',
]);
export type Sector = z.infer<typeof SectorSchema>;

export const CompanyMasterSchema = z.object({
    code: z.string(), // e.g. "1605"
    name: z.string(),
    name_en: z.string(),
    sector: SectorSchema,
    market: MarketSchema,
    listing_date: z.string(), // YYYY-MM-DD
    settled_month: z.number(), // 決算月
});

export type CompanyMaster = z.infer<typeof CompanyMasterSchema>;

// ─── Graph Node Definition (FalkorDB スキーマ用) ────────────────────────────────

export const NodeDefinitionSchema = z.object({
    label: z.string(), // e.g. "Company"
    properties: z.record(
        z.string(),
        z.object({
            type: z.enum(['string', 'integer', 'float', 'boolean']),
            required: z.boolean(),
            indexed: z.boolean(),
        })
    ),
});

export type NodeDefinition = z.infer<typeof NodeDefinitionSchema>;

// Company ノード定義の例
export const COMPANY_NODE_DEF: NodeDefinition = {
    label: 'Company',
    properties: {
        code: { type: 'string', required: true, indexed: true },
        name: { type: 'string', required: true, indexed: false },
        sector: { type: 'string', required: true, indexed: true },
        market: { type: 'string', required: true, indexed: false },
    },
};

// Sector ノード定義の例
export const SECTOR_NODE_DEF: NodeDefinition = {
    label: 'Sector',
    properties: {
        name: { type: 'string', required: true, indexed: true },
    },
};

// ─── Edge Definition (因果関係テンプレート) ─────────────────────────────────────

export const EdgeDefinitionSchema = z.object({
    type: z.string(), // e.g. "BELONGS_TO"
    from_label: z.string(),
    to_label: z.string(),
    properties: z.record(
        z.string(),
        z.object({
            type: z.enum(['string', 'integer', 'float', 'boolean']),
            required: z.boolean(),
        })
    ),
});

export type EdgeDefinition = z.infer<typeof EdgeDefinitionSchema>;

// BELONGS_TO エッジ定義の例（Company → Sector）
export const BELONGS_TO_EDGE_DEF: EdgeDefinition = {
    type: 'BELONGS_TO',
    from_label: 'Company',
    to_label: 'Sector',
    properties: {
        weight: { type: 'float', required: false },
    },
};

// ─── Schema State ─────────────────────────────────────────────────────────────

export const SchemaStateSchema = z.object({
    initialized: z.boolean(),
    node_definitions: z.array(NodeDefinitionSchema),
    edge_definitions: z.array(EdgeDefinitionSchema),
    last_updated: z.string().optional(), // ISO 8601 timestamp
});

export type SchemaState = z.infer<typeof SchemaStateSchema>;

// ─── Data Ingestion State ─────────────────────────────────────────────────────

export const IngestionStateSchema = z.enum([
    'idle',
    'loading_master',
    'validating',
    'confirming_write',
    'writing',
    'done',
    'error',
]);

export type IngestionState = z.infer<typeof IngestionStateSchema>;

export const IngestionContextSchema = z.object({
    state: IngestionStateSchema,
    companies: z.array(CompanyMasterSchema).default([]),
    error_message: z.string().optional(),
    /** fact_write 確認時に表示する概要（"123 companies, 11 sectors"） */
    summary: z.string().optional(),
});

export type IngestionContext = z.infer<typeof IngestionContextSchema>;

// ─── ctx-ontology Props ──────────────────────────────────────────────────────

export interface OntologyPanelProps {
    /** fact_write 確認後のコールバック（スキーマ更新・データロード完了） */
    onIngestionComplete: () => void;
    /** エラー発生時のコールバック */
    onIngestionError: (message: string) => void;
}