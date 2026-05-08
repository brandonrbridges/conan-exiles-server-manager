import { Toaster } from '@/components/ui/sonner'
import { TooltipProvider } from '@/components/ui/tooltip'
import { ThemeProvider, useTheme } from '@/lib/theme'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { type ReactNode, useState } from 'react'

export function Providers({ children }: { children: ReactNode }) {
	const [queryClient] = useState(
		() =>
			new QueryClient({
				defaultOptions: {
					queries: {
						staleTime: 5_000,
						retry: false,
						refetchOnWindowFocus: false,
					},
				},
			}),
	)

	return (
		<QueryClientProvider client={queryClient}>
			<ThemeProvider>
				<TooltipProvider delayDuration={200}>
					{children}
					<ThemedToaster />
				</TooltipProvider>
			</ThemeProvider>
		</QueryClientProvider>
	)
}

/**
 * Toaster wrapper that follows the live theme. Lives inside
 * `ThemeProvider` so it can subscribe — the provider applies the
 * `.dark` class to `<html>`, but sonner needs the resolved theme as
 * a prop to pick the right CSS variables.
 */
function ThemedToaster() {
	const { resolvedTheme } = useTheme()
	return <Toaster theme={resolvedTheme} position="bottom-right" richColors closeButton />
}
