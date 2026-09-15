import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import UnoCSS from 'unocss/vite'
import { resolve } from 'path'

export function vendorChunkName(id: string): string | undefined {
  if (!id.includes('node_modules')) return undefined
  if (id.includes('vue')) return 'vendor-vue'
  return 'vendor'
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    UnoCSS()
  ],
  base: './',
  build: {
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      // [Phase 5.1] Fail-fast: Cắt đứt mọi liên kết vô tình với Node.js API trong Frontend
      external: ['fs', 'path', 'os', 'crypto', 'child_process'],
      input: {
        index: resolve(__dirname, 'index.html'),
        widget: resolve(__dirname, 'widget.html'),
        dashboard: resolve(__dirname, 'dashboard.html'),
        setup: resolve(__dirname, 'setup.html'),
      },
      output: {
        manualChunks: vendorChunkName
      }
    }
  },
  server: {
    host: true, // Listen on all local IPs (0.0.0.0) for Mobile LAN access
    port: 5173,
    strictPort: true,
    proxy: {
      '/ws': {
        target: 'http://127.0.0.1:8002',
        ws: true,
        changeOrigin: true
      },
      '/v1': {
        target: 'http://127.0.0.1:8002',
        changeOrigin: true
      }
    }
  }
})
