import { defineConfig } from '@playwright/test';

export default defineConfig({
    testDir: './docs/specs',
    testMatch: '**/*.spec.ts',
    outputDir: './evidence',
    use: {
        baseURL: 'http://localhost:1420',
        screenshot: 'on',
    },
    webServer: {
        command: 'pnpm dev',
        url: 'http://localhost:1420',
        reuseExistingServer: true,
    },
});