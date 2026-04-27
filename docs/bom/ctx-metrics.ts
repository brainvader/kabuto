/**
 * @file docs/bom/ctx-metrics.ts
 * @description ctx-metrics — Polars計算・チャート描画・レポート読み込み
 *
 * 責務:
 * - Polars（Wasm または Rust via Tauri）で Parquet ファイルを読み込み
 * - global-store.selection と reports/ のシミュレーション結果を組み合わせて計算
 * - Recharts でチャート描画
 * - operationPolicy: read → auto（UI表示のみ、DB書き込みなし）
 */

import { z } from 'zod';

// ─── Metrics Data Types ──────────────────────────────────────────────────────

export const PriceDataSchema = z.object({
    date: z.string(), // YYYY-MM-DD
    close: z.number(),
    open: z.number(),
    high: z.number(),
    low: z.number(),
    volume: z.number(),
});

export type PriceData = z.infer<typeof PriceDataSchema>;

export const FinancialMetricsSchema = z.object({
    code: z.string(),
    fiscal_year: z.number(),
    revenue: z.number(),
    operating_income: z.number(),
    net_income: z.number(),
    eps: z.number(),
    roe: z.number(),
    roa: z.number(),
});

export type FinancialMetrics = z.infer<typeof FinancialMetricsSchema>;

// ─── Simulation Report結果 ──────────────────────────────────────────────────────

export const SimulationResultSchema = z.object({
    id: z.string(),
    timestamp: z.string(),
    action: z.string(), // e.g. "stress_test", "valuation_dcf"
    selection_id: z.string(),
    report_path: z.string(),
    /** JSON で返されたシミュレーション結果 */
    data: z.record(z.string(), z.unknown()),
});

export type SimulationResult = z.infer<typeof SimulationResultSchema>;

// ─── Chart Data（Recharts用） ────────────────────────────────────────────────

export const ChartDataPointSchema = z.object({
    x: z.union([z.string(), z.number()]), // date or numeric
    y: z.number(),
    label: z.string().optional(),
});

export type ChartDataPoint = z.infer<typeof ChartDataPointSchema>;

export const ChartConfigSchema = z.object({
    type: z.enum(['line', 'bar', 'area', 'scatter']),
    title: z.string(),
    xLabel: z.string(),
    yLabel: z.string(),
    data: z.array(ChartDataPointSchema),
});

export type ChartConfig = z.infer<typeof ChartConfigSchema>;

// ─── Portfolio Analysis ──────────────────────────────────────────────────────

export const PortfolioMetricsSchema = z.object({
    total_value: z.number(),
    top_holdings: z.array(
        z.object({
            code: z.string(),
            name: z.string(),
            value: z.number(),
            weight_pct: z.number(),
        })
    ),
    sector_distribution: z.record(z.string(), z.number()),
    risk_score: z.number(),
});

export type PortfolioMetrics = z.infer<typeof PortfolioMetricsSchema>;

// ─── Cached Data State ──────────────────────────────────────────────────────

export const MetricsStateSchema = z.object({
    selection_id: z.string().optional(),
    price_data: z.array(PriceDataSchema).default([]),
    financial_metrics: z.array(FinancialMetricsSchema).default([]),
    simulation_results: z.array(SimulationResultSchema).default([]),
    portfolio: PortfolioMetricsSchema.optional(),
    last_loaded: z.string().optional(), // ISO 8601
});

export type MetricsState = z.infer<typeof MetricsStateSchema>;

// ─── Data Source Configs ─────────────────────────────────────────────────────

export const DataSourceSchema = z.object({
    type: z.enum(['parquet', 'json', 'csv']),
    path: z.string(),
    cache_ttl_seconds: z.number().optional(),
});

export type DataSource = z.infer<typeof DataSourceSchema>;

export const DataSourceRegistrySchema = z.record(z.string(), DataSourceSchema);
export type DataSourceRegistry = z.infer<typeof DataSourceRegistrySchema>;

// ─── Polars Query Builder ────────────────────────────────────────────────────

export const PolarsQuerySchema = z.object({
    source: z.string(), // data source key
    select: z.array(z.string()),
    filter: z.record(z.string(), z.unknown()).optional(),
    group_by: z.array(z.string()).optional(),
    order_by: z.array(z.string()).optional(),
    limit: z.number().optional(),
});

export type PolarsQuery = z.infer<typeof PolarsQuerySchema>;

// ─── ctx-metrics Props ───────────────────────────────────────────────────────

export interface MetricsPanelProps {
    /** selection が変更されたときのコールバック */
    onSelectionChange: (selectionId: string) => void;
    /** レポートファイルパスが ctx-aip から渡されたときのコールバック */
    onReportLoaded: (reportPath: string) => void;
}