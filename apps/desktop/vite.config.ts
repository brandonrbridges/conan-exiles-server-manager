import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import type { UserConfig } from 'vite'
import { defineConfig } from 'vitest/config'

const host = process.env.TAURI_DEV_HOST

const server: UserConfig['server'] = {
	port: 1420,
	strictPort: true,
	watch: { ignored: ['**/src-tauri/**'] },
}

if (host) {
	server.host = host
	server.hmr = { protocol: 'ws', host, port: 1421 }
}

export default defineConfig({
	plugins: [react(), tailwindcss()],
	resolve: {
		alias: {
			'@': path.resolve(__dirname, './src'),
		},
	},
	clearScreen: false,
	server,
	test: {
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./src/test-setup.ts'],
	},
})
