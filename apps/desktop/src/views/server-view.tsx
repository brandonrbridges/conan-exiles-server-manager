import { DeleteServerDialog } from '@/components/delete-server-dialog'
import { ServerFormDialog } from '@/components/server-form-dialog'
import { StatusDot } from '@/components/status-dot'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { useActiveConnection } from '@/hooks/use-active-connection'
import type { ConnectionState, Server } from '@/types/generated'
import { Pencil, Power, Trash2 } from 'lucide-react'
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
					<p className="text-sm text-muted-foreground">{STATE_COPY[state]}</p>
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

			<main className="flex-1 px-8 py-6">
				<Card>
					<CardContent className="flex flex-col gap-4 p-6">
						<div className="grid grid-cols-2 gap-4">
							<DetailRow label="Host" value={server.host} />
							<DetailRow label="RCON port" value={String(server.rcon_port)} />
							<DetailRow
								label="Admin password"
								value={server.has_admin_pw ? 'Stored' : 'Not set'}
							/>
							<DetailRow
								label="Created"
								value={new Date(Number(server.created_at) * 1000).toLocaleString()}
							/>
						</div>

						<Separator />

						<div className="flex items-center justify-between">
							<div className="flex items-center gap-3">
								<Power className="size-4 text-muted-foreground" />
								<div className="flex flex-col">
									<span className="text-sm font-medium">Connection</span>
									<span className="text-xs text-muted-foreground">
										Opens automatically while this server is selected.
									</span>
								</div>
							</div>
							<div className="flex items-center gap-2">
								<StatusDot state={state} />
								<span className="text-sm">{STATE_COPY[state]}</span>
							</div>
						</div>
					</CardContent>
				</Card>

				<p className="mt-6 text-xs text-muted-foreground">
					Live admin (players, broadcast, console) lands in the next release.
				</p>
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

function DetailRow({ label, value }: { label: string; value: string }) {
	return (
		<div className="flex flex-col gap-1">
			<span className="text-xs uppercase tracking-wide text-muted-foreground">{label}</span>
			<span className="text-sm font-medium">{value}</span>
		</div>
	)
}
