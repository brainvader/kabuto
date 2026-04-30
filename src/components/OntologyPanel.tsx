/**
 * @file src/components/OntologyPanel.tsx
 * @context ctx-ontology
 * @bom docs/bom/ctx-ontology.ts
 */

import { useState } from 'react';
import { Loader2, CheckCircle, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { IngestionState, OntologyPanelProps } from '../../docs/bom/ctx-ontology';

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
        setSummary('3 companies, 3 sectors');
        setState('confirming_write');
    };

    const handleConfirm = async () => {
        setState('writing');
        await new Promise((r) => setTimeout(r, 1200));
        setState('done');
        onIngestionComplete();
    };

    const handleCancel = () => {
        setState('idle');
        setSummary('');
    };

    const handleRetry = () => setState('idle');

    return (
        <div className="rounded-md border border-border bg-card p-4 text-card-foreground">
            <div className="mb-4 flex items-center justify-between">
                <h3 className="text-sm font-semibold">01 · Ontology Explorer</h3>
                <span className="rounded bg-muted px-2 py-1 text-xs">fact_write</span>
            </div>

            <div className="mb-4 flex items-center gap-2 text-xs">
                <span className="font-mono text-muted-foreground">STATE</span>
                {state === 'idle' && <span className="text-green-500">● idle</span>}
                {(state === 'loading_master' || state === 'validating') && <span className="text-amber-500">● {state}</span>}
                {state === 'confirming_write' && <span className="text-amber-500">● confirming_write</span>}
                {state === 'writing' && <span className="text-amber-500">● writing</span>}
                {state === 'done' && <span className="text-green-500">● done</span>}
                {state === 'error' && <span className="text-destructive">● error</span>}
            </div>

            {state === 'idle' && (
                <Button onClick={handleStart}>↻ 手動更新</Button>
            )}

            {(state === 'loading_master' || state === 'validating') && (
                <div className="flex flex-col items-center gap-3 py-4">
                    <Loader2 className="animate-spin text-primary" />
                    <p className="text-sm">ロード中...</p>
                </div>
            )}

            {state === 'confirming_write' && (
                <div className="rounded border border-border bg-background p-4">
                    <h4 className="mb-2 text-xs font-semibold uppercase">書き込み確認</h4>
                    <p className="mb-1 font-mono text-sm">{summary}</p>
                    <p className="mb-4 text-xs text-muted-foreground">この操作は取り消しできません。</p>
                    <div className="flex gap-2">
                        <Button variant="outline" onClick={handleCancel}>キャンセル</Button>
                        <Button variant="destructive" onClick={handleConfirm}>確認して書き込み</Button>
                    </div>
                </div>
            )}

            {state === 'writing' && (
                <div className="flex flex-col items-center gap-3 py-4">
                    <Loader2 className="animate-spin text-primary" />
                    <p className="text-sm">書き込み中...</p>
                </div>
            )}

            {state === 'done' && (
                <div className="flex flex-col items-center gap-3 py-4">
                    <CheckCircle className="text-green-500" />
                    <p className="text-sm">完了しました</p>
                </div>
            )}

            {state === 'error' && (
                <>
                    <div className="mb-3 flex gap-3 rounded border border-destructive bg-destructive/10 p-3">
                        <AlertCircle className="shrink-0 text-destructive" />
                        <p className="text-sm">{errorMessage}</p>
                    </div>
                    <Button variant="outline" onClick={handleRetry}>リトライ</Button>
                </>
            )}
        </div>
    );
};