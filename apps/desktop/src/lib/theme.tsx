import { commands } from '@/lib/commands'
import {
	type ReactNode,
	createContext,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useState,
} from 'react'

export type Theme = 'system' | 'light' | 'dark'

const STORAGE_KEY = 'theme'

interface ThemeContextValue {
	theme: Theme
	setTheme: (next: Theme) => void
	resolvedTheme: 'light' | 'dark'
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

function readSystemPreference(): 'light' | 'dark' {
	if (typeof window === 'undefined') return 'dark'
	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function isTheme(value: unknown): value is Theme {
	return value === 'system' || value === 'light' || value === 'dark'
}

function applyTheme(resolved: 'light' | 'dark') {
	const root = document.documentElement
	root.classList.toggle('dark', resolved === 'dark')
	root.dataset.theme = resolved
}

export function ThemeProvider({ children }: { children: ReactNode }) {
	// Default to `system` until the saved value lands. Re-renders are cheap
	// here — this state changes ~once per session.
	const [theme, setThemeState] = useState<Theme>('system')
	const [systemPref, setSystemPref] = useState<'light' | 'dark'>(readSystemPreference)

	// Hydrate from storage on mount.
	useEffect(() => {
		let cancelled = false
		commands
			.getSetting(STORAGE_KEY)
			.then((value) => {
				if (cancelled) return
				if (isTheme(value)) setThemeState(value)
			})
			.catch(() => {
				// Storage failures shouldn't block the UI — fall back to system.
			})
		return () => {
			cancelled = true
		}
	}, [])

	// Watch for OS-level theme changes when in `system` mode.
	useEffect(() => {
		const mql = window.matchMedia('(prefers-color-scheme: dark)')
		const onChange = () => setSystemPref(mql.matches ? 'dark' : 'light')
		mql.addEventListener('change', onChange)
		return () => mql.removeEventListener('change', onChange)
	}, [])

	const resolvedTheme: 'light' | 'dark' = theme === 'system' ? systemPref : theme

	useEffect(() => {
		applyTheme(resolvedTheme)
	}, [resolvedTheme])

	const setTheme = useCallback((next: Theme) => {
		setThemeState(next)
		commands.setSetting(STORAGE_KEY, next).catch(() => {
			// Best-effort persistence; the in-memory value still applies.
		})
	}, [])

	const value = useMemo<ThemeContextValue>(
		() => ({ theme, setTheme, resolvedTheme }),
		[theme, setTheme, resolvedTheme],
	)

	return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}

export function useTheme(): ThemeContextValue {
	const ctx = useContext(ThemeContext)
	if (!ctx) throw new Error('useTheme must be used within <ThemeProvider>')
	return ctx
}
