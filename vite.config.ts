import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";
import { storybookTest } from "@storybook/addon-vitest/vitest-plugin";
import { playwright } from "@vitest/browser-playwright";

const host = process.env.TAURI_DEV_HOST;
const isPlaywright = process.env.VITE_PLAYWRIGHT === 'true';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  optimizeDeps: {
    force: isPlaywright,
  },
  resolve: {
    alias: [
      ...(isPlaywright ? [
        { find: '@tauri-apps/api/core', replacement: path.resolve(__dirname, './src/__mocks__/api-core.ts') },
      ] : []),
      { find: '@components', replacement: path.resolve(__dirname, './src/components') },
      { find: '@hooks', replacement: path.resolve(__dirname, './src/hooks') },
      { find: '@', replacement: path.resolve(__dirname, './src') },
    ],
  },
  test: {
    projects: [
      {
        extends: './vite.config.ts',
        test: {
          name: 'unit',
          environment: 'jsdom',
          globals: true,
          setupFiles: ['./src/test-setup.ts'],
          exclude: ['**/*.spec.ts', '**/node_modules/**'],
          include: ['src/**/*.test.{ts,tsx}'],
        },
      },
      {
        plugins: [storybookTest({ configDir: './.storybook' })],
        test: {
          name: 'storybook',
          browser: {
            enabled: true,
            headless: true,
            provider: playwright(),
            instances: [{ browser: 'chromium' }],
          },
        },
      },
    ],
  },
});