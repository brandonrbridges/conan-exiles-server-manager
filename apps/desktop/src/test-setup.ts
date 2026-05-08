import '@testing-library/jest-dom/vitest'

// jsdom doesn't ship matchMedia. The theme provider polls
// `prefers-color-scheme: dark` to resolve the `system` setting; stub it
// here so any test that mounts the provider gets a sensible default.
if (typeof window !== 'undefined' && !window.matchMedia) {
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: (query: string) => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: () => {},
			removeListener: () => {},
			addEventListener: () => {},
			removeEventListener: () => {},
			dispatchEvent: () => false,
		}),
	})
}
