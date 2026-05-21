import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import { visualizer } from 'rollup-plugin-visualizer'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), visualizer({ open: false, filename: 'stats.html' })],
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
            if (id.includes('pdfjs-dist')) {
              return 'pdf-vendor';
            }
            if (id.includes('@marp-team')) {
              return 'marp-vendor';
            }
            if (id.includes('react-dom') || id.includes('node_modules/react/') || id.includes('scheduler')) {
              return 'react-vendor';
            }
            if (id.includes('recharts') || id.includes('d3') || id.includes('victory-vendor')) {
              return 'recharts-vendor';
            }
            if (
              id.includes('react-markdown') || 
              id.includes('remark') || 
              id.includes('rehype') || 
              id.includes('micromark') || 
              id.includes('unist') || 
              id.includes('vfile') || 
              id.includes('hast') || 
              id.includes('mdast') ||
              id.includes('unified') ||
              id.includes('property-information') || 
              id.includes('space-separated-tokens') || 
              id.includes('comma-separated-tokens')
            ) {
              return 'markdown-vendor';
            }
            if (id.includes('lucide-react')) {
              return 'lucide-vendor';
            }
            if (
              id.includes('radix-ui') || 
              id.includes('@radix-ui') || 
              id.includes('@base-ui') ||
              id.includes('cmdk') ||
              id.includes('embla-carousel') ||
              id.includes('vaul') ||
              id.includes('sonner') ||
              id.includes('input-otp') ||
              id.includes('react-day-picker') ||
              id.includes('react-resizable-panels')
            ) {
              return 'ui-vendor';
            }
          }
        },
      },
    },
    chunkSizeWarningLimit: 2000,
  },
  server: {
    host: '127.0.0.1',
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        secure: false,
      },
    },
  },
})
