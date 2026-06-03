import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    // Generate a single JS file and a single CSS file
    rollupOptions: {
      output: {
        entryFileNames: 'assets/index.js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]'
      }
    },
    // Inline small assets
    assetsInlineLimit: 4096,
    // Minify for production
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
      }
    }
  }
})
