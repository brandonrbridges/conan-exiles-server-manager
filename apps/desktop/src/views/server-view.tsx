import { BroadcastComposer } from '@/components/broadcast-composer'
import { DeleteServerDialog } from '@/components/delete-server-dialog'
import { PlayerTable } from '@/components/player-table'
import { ServerFormDialog } from '@/components/server-form-dialog'
import { ServerHeroStats } from '@/components/server-hero-stats'
import { StatusDot } from '@/components/status-dot'
import { Button } from '@/components/ui/button'
import { useActiveConnection } from '@/hooks/use-active-connection'
import type { ConnectionState, Server } from '@/types/generated'
import { Pencil, Trash2 } from 'lucide-react'
import { useState } from 'react'

const STATE_COPY: Record<ConnectionState, string> = {
	disconnected: 'Disconnected',
	connecting: 'Connecting…',
	open: 'Connected',
	reconnecting: 'Reconnecting…',
	failed: 'Authentication failed',
}

interface ServerViewProps {
	server: Server
	onDeleted: () => void
}

export function ServerView({ server, onDeleted }: ServerViewProps) {
	const conn = useActiveConnection(server.id)
	const state: ConnectionState = conn.data ?? 'disconnected'

	const [editOpen, setEditOpen] = useState(false)
	const [deleteOpen, setDeleteOpen] = useState(false)

	return (
		<div className="flex h-screen flex-col overflow-y-auto">
			<header className="flex items-start justify-between gap-4 border-b border-border px-8 py-6">
				<div className="flex flex-col gap-2">
					<div className="flex items-center gap-3">
						<StatusDot state={state} className="size-3" />
						<h1 className="text-2xl font-semibold tracking-tight">{server.name}</h1>
					</div>
					<p className="text-sm text-muted-foreground">
						{server.host}:{server.rcon_port} · {STATE_COPY[state]}
					</p>
				</div>
				<div className="flex gap-2">
					<Button variant="outline" size="sm" onClick={() => setEditOpen(true)}>
						<Pencil className="size-4" />
						Edit
					</Button>
					<Button variant="outline" size="sm" onClick={() => setDeleteOpen(true)}>
						<Trash2 className="size-4" />
						Delete
					</Button>
				</div>
			</header>

			<main className="flex flex-1 flex-col gap-6 px-8 py-6">
				{state === 'open' ? (
					<>
						<ServerHeroStats serverId={server.id} />
						<div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
							<div className="lg:col-span-2">
								<PlayerTable serverId={server.id} />
							</div>
							<div className="flex flex-col gap-4">
								<BroadcastComposer serverId={server.id} />
							</div>
						</div>
					</>
				) : (
					<NotReadyNotice state={state} />
				)}
			</main>

			<ServerFormDialog open={editOpen} onOpenChange={setEditOpen} server={server} />
			<DeleteServerDialog
				open={deleteOpen}
				onOpenChange={setDeleteOpen}
				server={server}
				onDeleted={onDeleted}
			/>
		</div>
	)
}

function NotReadyNotice({ state }: { state: ConnectionState }) {
	const copy: Partial<Record<ConnectionState, string>> = {
		disconnected: 'Open this server to connect.',
		connecting: 'Connecting to the server… player list will appear once the handshake completes.',
		reconnecting: 'Reconnecting to the server. Will resume automatically.',
		failed:
			'The server rejected this RCON password. Edit the server and re-enter the password to continue.',
	}
	return (
		<div className="rounded-md border border-dashed border-border bg-muted/40 p-6 text-sm text-muted-foreground">
			{copy[state] ?? 'Standing by…'}
		</div>
	)
}
