import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { ConnectionState } from '@/types/generated'

const COLOURS: Record<ConnectionState, string> = {
	disconnected: 'bg-muted-foreground/40',
	connecting: 'bg-amber-500 animate-pulse',
	open: 'bg-emerald-500',
	reconnecting: 'bg-amber-500 animate-pulse',
	failed: 'bg-red-500',
}

const LABELS: Record<ConnectionState, string> = {
	disconnected: 'Disconnected',
	connecting: 'Connecting…',
	open: 'Connected',
	reconnecting: 'Reconnecting…',
	failed: 'Connection failed',
}

interface StatusDotProps {
	state: ConnectionState
	className?: string
}

export function StatusDot({ state, className }: StatusDotProps) {
	return (
		<Tooltip>
			<TooltipTrigger asChild>
				<span
					aria-label={LABELS[state]}
					className={cn('inline-block size-2 rounded-full', COLOURS[state], className)}
				/>
			</TooltipTrigger>
			<TooltipContent side="right">{LABELS[state]}</TooltipContent>
		</Tooltip>
	)
}
