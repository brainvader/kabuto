/**
 * Slot 1: 発注用ヘッダー (JSDoc Metadata)
 *
 * @context ctx-ontology / OntologyPanel
 * @bom docs/bom/ctx-ontology.ts
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
 */

/**
 * Slot 2: 外部依存のインポート (Imports)
 */
import { test, expect } from '@playwright/test';

/**
 * Slot 4: 挙動の検証コード (Story Verification)
 */

test.describe('ctx-ontology Visual Story', () => {
    test('should satisfy the story steps with evidence', async ({ page }) => {

        await page.goto('/');

        // Step 1: idle
        await expect(page.getByRole('button', { name: /手動更新/ })).toBeVisible();
        await page.screenshot({ path: 'evidence/ctx-ontology_01_idle.png' });

        // Step 2-3: loading_master
        await page.getByRole('button', { name: /手動更新/ }).click();
        await expect(page.getByText(/ロード中/)).toBeVisible();
        await page.screenshot({ path: 'evidence/ctx-ontology_02_loading.png' });

        // Step 4: confirming_write
        await expect(page.getByText(/3 companies, 3 sectors/)).toBeVisible({ timeout: 10000 });
        await page.screenshot({ path: 'evidence/ctx-ontology_03_confirming.png' });

        // Step 5-6: writing
        await page.getByRole('button', { name: /確認して書き込み/ }).click();
        await expect(page.getByText(/書き込み中/)).toBeVisible();
        await page.screenshot({ path: 'evidence/ctx-ontology_04_writing.png' });

        // Step 7: done
        await expect(page.getByText(/完了しました/)).toBeVisible({ timeout: 15000 });
        await page.screenshot({ path: 'evidence/ctx-ontology_05_done.png' });
    });
});