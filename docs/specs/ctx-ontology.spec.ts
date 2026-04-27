/**
 * Slot 1: 発注用ヘッダー (JSDoc Metadata)
 *
 * @context ctx-ontology / OntologyPanel
 * @bom docs/bom/ctx-ontology.ts
 * @state docs/bom/ctx-ontology.state.md
 *
 * @story
 * 1. ユーザーが「手動更新」ボタンをクリックする
 * 2. UIが loading_master 状態になり、ローディング表示に切り替わる
 * 3. JPX CSV のダウンロード・バリデーションが完了する
 * 4. UIが confirming_write 状態になり、サマリー（"X companies, Y sectors"）と確認ダイアログを表示する
 * 5. ユーザーが「確認して書き込み」ボタンをクリックする
 * 6. UIが writing 状態になり、FalkorDB への書き込みが実行される
 * 7. 完了後、UIが done 状態になり、成功メッセージを表示する
 * 8. onIngestionComplete() コールバックが呼ばれる
 *
 * @output src/components/OntologyPanel.tsx
 * @output src/hooks/useIngestion.ts
 */

/**
 * Slot 2: 外部依存のインポート (Imports)
 */
import { test, expect } from '@playwright/test';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import type {
    IngestionContext,
    IngestionState,
    CompanyMaster,
    OntologyPanelProps,
} from '../../docs/bom/ctx-ontology';
import { OntologyPanel } from '@/components/OntologyPanel';

/**
 * Slot 3: モック・セットアップ (Test Setup)
 * 実装と検証を切り離すための「独立した基準器」。
 */
const setupMock = () => {
    const mockCompanies: CompanyMaster[] = [
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
    ];

    const mockIngestionContext: IngestionContext = {
        state: 'idle',
        companies: [],
        error_message: undefined,
        summary: undefined,
    };

    const mockProps: OntologyPanelProps = {
        onIngestionComplete: vi.fn(),
        onIngestionError: vi.fn(),
    };

    return { mockCompanies, mockIngestionContext, mockProps };
};

/**
 * Slot 4: 挙動の検証コード (Story Verification)
 */

// --- 1. AIの内省 (Logic Verification) ---

test('logic: idle 状態で「手動更新」ボタンが表示される', () => {
    const { mockProps } = setupMock();
    render(<OntologyPanel { ...mockProps } />);
    expect(screen.getByRole('button', { name: /手動更新/ })).toBeInTheDocument();
});

test('logic: confirming_write でサマリーと確認ボタンが表示される', () => {
    const { mockProps } = setupMock();
    // confirming_write 状態を注入するためのモック
    render(<OntologyPanel { ...mockProps } initialState = "confirming_write" summary = "2 companies, 2 sectors" />);
    expect(screen.getByText(/2 companies, 2 sectors/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /確認して書き込み/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /キャンセル/ })).toBeInTheDocument();
});

test('logic: キャンセルで idle に戻る', async () => {
    const { mockProps } = setupMock();
    render(<OntologyPanel { ...mockProps } initialState = "confirming_write" summary = "2 companies, 2 sectors" />);
    fireEvent.click(screen.getByRole('button', { name: /キャンセル/ }));
    await waitFor(() => {
        expect(screen.getByRole('button', { name: /手動更新/ })).toBeInTheDocument();
    });
});

test('logic: done 状態で onIngestionComplete が呼ばれる', async () => {
    const { mockProps } = setupMock();
    render(<OntologyPanel { ...mockProps } initialState = "done" />);
    await waitFor(() => {
        expect(mockProps.onIngestionComplete).toHaveBeenCalledTimes(1);
    });
});

test('logic: error 状態でエラーメッセージと「リトライ」ボタンが表示される', () => {
    const { mockProps } = setupMock();
    render(<OntologyPanel { ...mockProps } initialState = "error" errorMessage = "ネットワークエラーが発生しました" />);
    expect(screen.getByText(/ネットワークエラーが発生しました/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /リトライ/ })).toBeInTheDocument();
});

// --- 2. 監督へのプレゼン (Visual Story) ---

test.describe('ctx-ontology Visual Story', () => {
    test('should satisfy the story steps with evidence', async ({ page }) => {
        await page.goto('http://localhost:1420');

        // Step 1: idle 状態 → 「手動更新」ボタンが存在する
        await expect(page.getByRole('button', { name: /手動更新/ })).toBeVisible();
        await page.screenshot({ path: 'evidence/ctx-ontology_01_idle.png' });

        // Step 2-3: ボタンクリック → loading_master → validating
        await page.getByRole('button', { name: /手動更新/ }).click();
        await expect(page.getByText(/ロード中/)).toBeVisible();
        await page.screenshot({ path: 'evidence/ctx-ontology_02_loading.png' });

        // Step 4: confirming_write → サマリーと確認ダイアログ表示
        await expect(page.getByText(/companies.*sectors/)).toBeVisible({ timeout: 10000 });
        await page.screenshot({ path: 'evidence/ctx-ontology_03_confirming.png' });

        // Step 5-6: 「確認して書き込み」クリック → writing
        await page.getByRole('button', { name: /確認して書き込み/ }).click();
        await expect(page.getByText(/書き込み中/)).toBeVisible();
        await page.screenshot({ path: 'evidence/ctx-ontology_04_writing.png' });

        // Step 7: done → 成功メッセージ表示
        await expect(page.getByText(/完了/)).toBeVisible({ timeout: 15000 });
        await page.screenshot({ path: 'evidence/ctx-ontology_05_done.png' });
    });
});