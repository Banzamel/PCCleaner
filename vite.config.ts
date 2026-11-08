import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

// Tauri expects a fixed port in development and plain ES2021 for WebView2
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'es2022', outDir: 'dist', emptyOutDir: true },
  test: { include: ['test/**/*.test.ts'], environment: 'node' },
})
