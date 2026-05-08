import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

// Mock the Tauri invoke surface — vitest runs in jsdom, where the real
// IPC bridge isn't available.
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(async (cmd: string) => {
		if (cmd === 'list_servers') return []
		return undefined
	}),
}))

import App from './App'

describe('App', () => {
	it('renders the empty state when no servers are saved', async () => {
		render(<App />)
		// Wait for the loading state to clear, then assert the empty-state CTA.
		expect(
			await screen.findByRole('button', { name: /add your first server/i }, { timeout: 3000 }),
		).toBeInTheDocument()
	})
})
