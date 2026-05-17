import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            if (id.includes('react-dom') || id.includes('react/') || id.includes('scheduler')) {
              return 'react-vendor';
            }
            if (id.includes('recharts') || id.includes('d3')) {
              return 'recharts-vendor';
            }
            if (id.includes('react-markdown') || id.includes('remark') || id.includes('micromark') || id.includes('unist') || id.includes('vfile') || id.includes('property-information') || id.includes('space-separated-tokens') || id.includes('comma-separated-tokens') || id.includes('hast')) {
              return 'markdown-vendor';
            }
            if (id.includes('lucide-react')) {
              return 'lucide-vendor';
            }
            if (id.includes('radix-ui') || id.includes('@radix-ui') || id.includes('@base-ui')) {
              return 'ui-vendor';
            }
            return 'vendor';
          }
        },
      },
    },
    chunkSizeWarningLimit: 600,
  },
  server: {
    host: '127.0.0.1',
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false,
      },
    },
  },
})
