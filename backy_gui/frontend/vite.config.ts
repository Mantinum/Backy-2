import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Vite configuration to include Tauri API modules for frontend usage
export default defineConfig({
  optimizeDeps: {
    include: [
      '@tauri-apps/api/core',
      '@tauri-apps/api/fs',
      '@tauri-apps/api/path'
    ]
  },
  base: './',
  plugins: [react()],
  build: {
    outDir: '../src-tauri/dist'
  }
})
