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
	build: {
		// Bundle ships with the desktop binary — no kB budget. Default 500
		// triggers on any non-trivial UI; raise so the warning surfaces real
		// growth (a chunk crossing 1MB is meaningful) rather than noise.
		chunkSizeWarningLimit: 1000,
		rollupOptions: {
			output: {
				// Split the heaviest framework dep out so no single chunk
				// crosses the 500kB warning threshold. Going more granular
				// (radix, query, forms separately) reintroduces circular
				// chunk warnings because of cross-dependencies.
				manualChunks: (id) => {
					if (id.match(/[\\/]node_modules[\\/](react|react-dom|scheduler)[\\/]/)) {
						return 'vendor-react'
					}
					if (id.includes('node_modules')) return 'vendor'
					return undefined
				},
			},
		},
	},
	test: {
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./src/test-setup.ts'],
	},
})
