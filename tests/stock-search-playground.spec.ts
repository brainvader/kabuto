import { test, expect } from '@playwright/test'

test.describe('StockSearch → PlayGround 統合', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/')
    })

    test('アプリが起動して各パネルが表示される', async ({ page }) => {
        await expect(page.getByTestId('stock-search-panel')).toBeVisible()
        await expect(page.getByTestId('playground-panel')).toBeVisible()
        await expect(page.getByTestId('metrics-panel')).toBeVisible()
        await page.screenshot({ path: 'evidence/e2e_01_initial.png' })
    })

    test('銘柄選択前は PlayGround にプレースホルダーが表示される', async ({ page }) => {
        await expect(page.getByTestId('playground-empty')).toBeVisible()
        await page.screenshot({ path: 'evidence/e2e_02_empty.png' })
    })

    test('銘柄を検索して選択すると PlayGround に company ノードが表示される', async ({ page }) => {
        await page.getByRole('searchbox').fill('INPEX')

        // モックが返す結果リストが表示されるまで待機
        await expect(page.getByTestId('stock-item-1605')).toBeVisible()
        await page.getByTestId('stock-item-1605').click()

        await expect(page.getByTestId('node-company')).toBeVisible()
        await page.screenshot({ path: 'evidence/e2e_03_company_node.png' })
    })
})