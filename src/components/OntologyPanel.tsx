import { useState } from 'react';
import { Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { IngestionState, OntologyPanelProps } from '../../docs/bom/ctx-ontology';

const Badge = ({ children, variant = 'muted' }: { children: React.ReactNode; variant?: 'green' | 'amber' | 'red' | 'muted' | 'blue' }) => {
    const styles = {
        green: 'bg-[var(--kabuto-emerald-dim)] text-[var(--kabuto-emerald)] border border-[#065f46]',
        amber: 'bg-[var(--kabuto-amber-dim)] text-[var(--kabuto-amber)] border border-[#78350f]',
        red: 'bg-[var(--kabuto-red-dim)] text-[var(--kabuto-red)] border border-[#7f1d1d]',
        muted: 'bg-[var(--kabuto-muted)] text-[var(--kabuto-fg-dim)] border border-[var(--kabuto-border)]',
        blue: 'bg-[var(--kabuto-accent-dim)] text-[var(--kabuto-accent)] border border-[#1e40af]',
    };
    return (
        <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-sm font-mono text-[10px] font-semibold tracking-widest uppercase ${styles[variant]}`}>
            {children}
        </span>
    );
};

const Dot = () => <span className="inline-block w-1.5 h-1.5 rounded-full bg-current" />;

const SchemaItem = ({ label, isEdge = false, count, extra }: { label: string; isEdge?: boolean; count: string; extra?: string }) => (
    <div className="flex items-center justify-between py-1.5 border-b border-[var(--kabuto-border)] last:border-0">
        <span className={`font-mono text-[11px] ${isEdge ? 'text-[var(--kabuto-fg-dim)]' : 'text-[var(--kabuto-violet)]'}`}>{label}</span>
        <div className="flex items-center gap-1.5">
            <Badge variant="muted">{count}</Badge>
            {extra && <span className="font-mono text-[10px] text-[var(--kabuto-fg-dim)]">{extra}</span>}
        </div>
    </div>
);

export const OntologyPanel = ({
    onIngestionComplete,
    onIngestionError,
    initialState,
    summary: testSummary,
    errorMessage: testError,
}: OntologyPanelProps & {
    initialState?: IngestionState;
    summary?: string;
    errorMessage?: string;
}) => {
    const [state, setState] = useState<IngestionState>(initialState ?? 'idle');
    const [summary, setSummary] = useState(testSummary ?? '');
    const [errorMessage] = useState(testError ?? '');

    const handleStart = async () => {
        setState('loading_master');
        await new Promise((r) => setTimeout(r, 800));
        setSummary('3,935 companies · 33 sectors · 3 markets');
        setState('confirming_write');
    };

    const handleConfirm = async () => {
        setState('writing');
        await new Promise((r) => setTimeout(r, 1200));
        setState('done');
        onIngestionComplete();
    };

    const handleCancel = () => { setState('idle'); setSummary(''); };
    const handleRetry = () => setState('idle');

    const stateBadge = () => {
        if (state === 'idle' || state === 'done') return <Badge variant="green"><Dot />{state}</Badge>;
        if (state === 'error') return <Badge variant="red"><Dot />error</Badge>;
        return <Badge variant="amber"><Dot />{state}</Badge>;
    };

    return (
        <div className="flex flex-col h-full bg-[var(--kabuto-card)] border border-[var(--kabuto-border)] rounded-sm text-[var(--kabuto-fg)] text-[13px]">

            {/* Panel Header */}
            <div className="flex items-center justify-between px-3.5 py-2 bg-[var(--kabuto-panel)] border-b border-[var(--kabuto-border)] shrink-0">
                <span className="font-mono text-[9px] font-bold tracking-[0.15em] uppercase text-[var(--kabuto-fg-dim)]">
                    01 · Ontology Explorer
                </span>
                <div className="flex items-center gap-1.5">
                    <Badge variant="amber">fact_write</Badge>
                    <Button
                        variant="outline"
                        size="sm"
                        className="h-6 px-2 font-mono text-[9px] tracking-wide border-[var(--kabuto-border)] text-[var(--kabuto-fg-dim)] hover:border-[var(--kabuto-accent)] hover:text-[var(--kabuto-accent)] bg-transparent"
                        onClick={handleStart}
                        disabled={state !== 'idle'}
                    >
                        ↻ 手動更新
                    </Button>
                </div>
            </div>

            {/* Panel Body */}
            <div className="flex-1 overflow-y-auto p-3">

                {/* Status Row */}
                <div className="flex items-center gap-2 py-2 border-b border-[var(--kabuto-border)] mb-3">
                    <span className="font-mono text-[9px] text-[var(--kabuto-fg-dim)] tracking-[0.08em]">STATE</span>
                    {stateBadge()}
                    <span className="font-mono text-[9px] text-[var(--kabuto-fg-dim)] tracking-[0.08em] ml-2">LAST UPDATE</span>
                    <span className="font-mono text-[10px] text-[var(--kabuto-fg-dim)]">2025-04-28</span>
                </div>

                {/* Loading */}
                {(state === 'loading_master' || state === 'validating') && (
                    <div className="flex flex-col items-center gap-3 py-5">
                        <Loader2 className="w-5 h-5 text-[var(--kabuto-accent)] animate-spin" />
                        <span className="font-mono text-[11px] text-[var(--kabuto-fg-dim)]">ロード中...</span>
                    </div>
                )}

                {/* Writing */}
                {state === 'writing' && (
                    <div className="flex flex-col items-center gap-3 py-5">
                        <Loader2 className="w-5 h-5 text-[var(--kabuto-emerald)] animate-spin" />
                        <span className="font-mono text-[11px] text-[var(--kabuto-fg-dim)]">書き込み中...</span>
                    </div>
                )}

                {/* Done */}
                {state === 'done' && (
                    <div className="flex flex-col items-center gap-3 py-5">
                        <span className="font-mono text-[11px] text-[var(--kabuto-emerald)]">✓ 完了しました</span>
                    </div>
                )}

                {/* Error */}
                {state === 'error' && (
                    <>
                        <div className="bg-[var(--kabuto-panel)] border border-[var(--kabuto-red)] rounded-sm p-3 mb-2.5">
                            <p className="text-[11px] text-[var(--kabuto-red)]">{errorMessage}</p>
                        </div>
                        <Button variant="outline" size="sm" onClick={handleRetry}
                            className="font-mono text-[9px] border-[var(--kabuto-border)] text-[var(--kabuto-fg-dim)] bg-transparent">
                            リトライ
                        </Button>
                    </>
                )}

                {/* Confirm Dialog */}
                {state === 'confirming_write' && (
                    <div className="bg-[var(--kabuto-panel)] border border-[var(--kabuto-amber)] rounded-sm p-3 mb-2.5">
                        <div className="font-mono text-[9px] text-[var(--kabuto-amber)] font-bold tracking-[0.1em] uppercase mb-1.5">
                            ⚠ fact_write · 確認必須
                        </div>
                        <p className="text-[11px] text-[var(--kabuto-fg-dim)] leading-relaxed mb-2.5">
                            JPX CSV からデータをインポートし FalkorDB に書き込みます。この操作は取り消せません。
                        </p>
                        <div className="font-mono text-[11px] text-[var(--kabuto-amber)] mb-2.5">{summary}</div>
                        <div className="grid grid-cols-[1fr_2fr] gap-2">
                            <Button variant="outline" size="sm" onClick={handleCancel}
                                className="font-mono text-[9px] bg-[var(--kabuto-red-dim)] text-[var(--kabuto-red)] border-[var(--kabuto-red)] hover:bg-[var(--kabuto-red-dim)]">
                                キャンセル
                            </Button>
                            <Button size="sm" onClick={handleConfirm}
                                className="font-mono text-[9px] bg-[var(--kabuto-emerald)] text-black font-bold hover:bg-[var(--kabuto-emerald)]/90">
                                確認して書き込み
                            </Button>
                        </div>
                    </div>
                )}

                {/* Graph Schema */}
                {(state === 'idle' || state === 'done') && (
                    <div className="bg-[var(--kabuto-panel)] border border-[var(--kabuto-border)] rounded-sm p-3 mb-2.5">
                        <div className="font-mono text-[10px] font-bold text-[var(--kabuto-fg)] mb-1 uppercase tracking-[0.08em]">Graph Schema</div>
                        <SchemaItem label="Company" count="3,935 nodes" extra="4 props" />
                        <SchemaItem label="Sector" count="33 nodes" extra="2 props" />
                        <SchemaItem label="Market" count="3 nodes" extra="1 props" />
                        <SchemaItem label="BELONGS_TO" isEdge count="3,935 edges" />
                    </div>
                )}

                {/* Object Mapping */}
                {(state === 'idle' || state === 'done') && (
                    <div className="bg-[var(--kabuto-panel)] border border-[var(--kabuto-border)] rounded-sm p-3 mb-2.5">
                        <div className="font-mono text-[10px] font-bold text-[var(--kabuto-fg)] mb-1 uppercase tracking-[0.08em]">Object Mapping</div>
                        <p className="text-[11px] text-[var(--kabuto-fg-dim)] leading-relaxed mb-2">
                            Parquet (株価履歴) / XBRL (財務) の絶対パスを URI プロパティとして紐付け
                        </p>
                        <div className="grid grid-cols-2 gap-1.5">
                            <div className="bg-[var(--kabuto-muted)] border border-[var(--kabuto-border)] rounded-sm p-2">
                                <div className="font-mono text-lg font-bold text-[var(--kabuto-emerald)] leading-none">3,812</div>
                                <div className="text-[9px] text-[var(--kabuto-fg-dim)] mt-1 uppercase tracking-[0.08em]">Parquet linked</div>
                            </div>
                            <div className="bg-[var(--kabuto-muted)] border border-[var(--kabuto-border)] rounded-sm p-2">
                                <div className="font-mono text-lg font-bold text-[var(--kabuto-accent)] leading-none">2,104</div>
                                <div className="text-[9px] text-[var(--kabuto-fg-dim)] mt-1 uppercase tracking-[0.08em]">XBRL linked</div>
                            </div>
                        </div>
                    </div>
                )}

            </div>
        </div>
    );
};