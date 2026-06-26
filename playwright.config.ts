import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: './tests',
    testMatch: '**/*.spec.ts',
    outputDir: './evidence',
    use: {
        baseURL: 'http://localhost:1420',
        launchOptions: {
            slowMo: 1000,
        },
    },
    webServer: {
        command: 'pnpm dev',
        url: 'http://localhost:1420',
        reuseExistingServer: !process.env.CI,
        env: { VITE_PLAYWRIGHT: 'true' },
        timeout: 60_000,
    },
});