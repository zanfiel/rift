import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3200',
        changeOrigin: true,
      },
      '/uploads': {
        target: 'http://localhost:3200',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:3200',
        ws: true,
      },
    },
  },
})
