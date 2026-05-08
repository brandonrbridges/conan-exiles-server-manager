import { commands } from '@/lib/commands'
import { cn } from '@/lib/utils'
import type { ConnectionState, Server } from '@/types/generated'
import { useQuery } from '@tanstack/react-query'
import { StatusDot } from './status-dot'

interface ServerSidebarItemProps {
	server: Server
	selected: boolean
	onSelect: () => void
}

const STATE_POLL_MS = 2_000

export function ServerSidebarItem({ server, selected, onSelect }: ServerSidebarItemProps) {
	// Each item polls its own connection state. Inactive servers always
	// resolve to `disconnected` from the registry, so the cost is small.
	const { data } = useQuery<ConnectionState>({
		queryKey: ['connectionState', server.id],
		queryFn: () => commands.connectionState(server.id),
		refetchInterval: STATE_POLL_MS,
		refetchIntervalInBackground: false,
	})

	const state: ConnectionState = data ?? 'disconnected'

	return (
		<li>
			<button
				type="button"
				onClick={onSelect}
				className={cn(
					'flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors',
					'hover:bg-accent hover:text-accent-foreground',
					selected && 'bg-accent text-accent-foreground',
				)}
			>
				<StatusDot state={state} />
				<span className="flex-1 truncate">{server.name}</span>
				<span className="text-xs text-muted-foreground">{server.rcon_port}</span>
			</button>
		</li>
	)
}
