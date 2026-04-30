/**
 * @file src/hooks/useIngestion.ts
 * @description ctx-ontology の状態管理・ビジネスロジック
 *
 * 責務:
 * - IngestionContext の状態遷移管理（idle → done または error）
 * - JPX CSV ダウンロード・バリデーション（simulation）
 * - fact_write 確認・書き込み処理
 * - エラーハンドリング・リトライ
 */

import { useState, useCallback, useEffect } from 'react';
import type {
    IngestionContext,
    IngestionState,
    CompanyMaster,
    OntologyPanelProps,
} from '../../docs/bom/ctx-ontology';

/**
 * useIngestion: JPXマスターデータ投入フロー管理
 *
 * @param onIngestionComplete コールバック
 * @param onIngestionError コールバック
 * @param initialState テスト用の初期状態
 * @returns { context, startIngestion, confirmWrite, cancelWrite, retry }
 */
export const useIngestion = (
    onIngestionComplete: OntologyPanelProps['onIngestionComplete'],
    onIngestionError: OntologyPanelProps['onIngestionError'],
    initialState?: IngestionState
) => {
    const [context, setContext] = useState<IngestionContext>({
        state: initialState || 'idle',
        companies: [],
        error_message: undefined,
        summary: undefined,
    });

    /**
     * ─── Step 1-3: 手動更新ボタン → ローディング → バリデーション
     *
     * @story Step 1: ユーザーが「手動更新」ボタンをクリックする
     * @story Step 2: UIが loading_master 状態になり、ローディング表示に切り替わる
     * @story Step 3: JPX CSV のダウンロード・バリデーションが完了する
     */
    const startIngestion = useCallback(async () => {
        // Step 1: ボタンクリック
        // Step 2: loading_master へ遷移
        setContext((prev) => ({
            ...prev,
            state: 'loading_master',
        }));

        try {
            // 実装例: JPX CSV をダウンロード
            // const response = await fetch('https://example.com/jpx-master.csv');
            // const csvData = await response.text();

            // シミュレーション: 1秒待機
            await new Promise((resolve) => setTimeout(resolve, 1000));

            // Step 3: validating 状態
            setContext((prev) => ({
                ...prev,
                state: 'validating',
            }));

            // シミュレーション: CSV パース・バリデーション
            const parsedCompanies = mockDownloadAndParse();

            // Step 4: confirming_write へ遷移
            // - バリデーション完了 → サマリー計算
            const uniqueSectors = new Set(parsedCompanies.map((c) => c.sector));
            const summary = `${parsedCompanies.length} companies, ${uniqueSectors.size} sectors`;

            setContext((prev) => ({
                ...prev,
                state: 'confirming_write',
                companies: parsedCompanies,
                summary,
            }));
        } catch (error) {
            // エラー発生
            const message = error instanceof Error ? error.message : 'Unknown error';
            setContext((prev) => ({
                ...prev,
                state: 'error',
                error_message: message,
            }));
            onIngestionError(message);
        }
    }, [onIngestionError]);

    /**
     * ─── Step 5-7: 確認ボタン → writing → done
     *
     * @story Step 5: ユーザーが「確認して書き込み」ボタンをクリックする
     * @story Step 6: UIが writing 状態になり、FalkorDB への書き込みが実行される
     * @story Step 7: 完了後、UIが done 状態になり、成功メッセージを表示する
     */
    const confirmWrite = useCallback(async () => {
        // Step 5: ボタンクリック
        // Step 6: writing へ遷移
        setContext((prev) => ({
            ...prev,
            state: 'writing',
        }));

        try {
            // シミュレーション: FalkorDB 書き込み（2秒）
            await new Promise((resolve) => setTimeout(resolve, 2000));

            // Step 7: done 状態
            setContext((prev) => ({
                ...prev,
                state: 'done',
            }));

            // Step 8: コールバック
            onIngestionComplete();
        } catch (error) {
            // エラー発生
            const message = error instanceof Error ? error.message : 'Write failed';
            setContext((prev) => ({
                ...prev,
                state: 'error',
                error_message: message,
            }));
            onIngestionError(message);
        }
    }, [onIngestionComplete, onIngestionError]);

    /**
     * ─── キャンセル: confirming_write → idle
     */
    const cancelWrite = useCallback(() => {
        setContext((prev) => ({
            ...prev,
            state: 'idle',
            companies: [],
            summary: undefined,
        }));
    }, []);

    /**
     * ─── リトライ: error → idle
     */
    const retry = useCallback(() => {
        setContext((prev) => ({
            ...prev,
            state: 'idle',
            error_message: undefined,
        }));
    }, []);

    // done 状態で onIngestionComplete を呼び出す（useEffect で監視）
    useEffect(() => {
        if (context.state === 'done') {
            // 既に startIngestion → confirmWrite 内で呼ばれているため、
            // ここでは追加呼び出しを避ける。
            // （テスト用の initialState='done' の場合のみ実行）
            if (initialState === 'done') {
                onIngestionComplete();
            }
        }
    }, [context.state, onIngestionComplete, initialState]);

    return {
        context,
        startIngestion,
        confirmWrite,
        cancelWrite,
        retry,
    };
};

/**
 * ─── モック関数 ───────────────────────────────────────────────────────────────
 *
 * 実装時は、以下を実際の API 呼び出しに置き換える：
 * - mockDownloadAndParse() → JPX API への fetch
 */

/**
 * mockDownloadAndParse: JPX CSV をダウンロード・パース（シミュレーション）
 */
function mockDownloadAndParse(): CompanyMaster[] {
    return [
        {
            code: '1605',
            name: 'INPEX',
            name_en: 'INPEX CORPORATION',
            sector: 'Energy',
            market: 'PrimeMarket',
            listing_date: '2006-04-14',
            settled_month: 12,
        },
        {
            code: '6758',
            name: 'ソニーグループ',
            name_en: 'SONY GROUP CORPORATION',
            sector: 'InformationTechnology',
            market: 'PrimeMarket',
            listing_date: '1958-12-01',
            settled_month: 3,
        },
        {
            code: '8306',
            name: 'SUMITOMO MITSUI FINANCIAL GROUP',
            name_en: 'SUMITOMO MITSUI FINANCIAL GROUP',
            sector: 'Financials',
            market: 'PrimeMarket',
            listing_date: '2001-04-01',
            settled_month: 3,
        },
    ];
}

export default useIngestion;