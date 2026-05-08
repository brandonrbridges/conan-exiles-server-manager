import { ServerFormDialog } from '@/components/server-form-dialog'
import { Sidebar } from '@/components/sidebar'
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty'
import { useServers } from '@/hooks/use-servers'
import { Providers } from '@/providers'
import { EmptyState } from '@/views/empty-state'
import { ServerView } from '@/views/server-view'
import { useEffect, useState } from 'react'

function AppShell() {
	const { data: servers = [], isLoading } = useServers()
	const [selectedId, setSelectedId] = useState<string | null>(null)
	const [addOpen, setAddOpen] = useState(false)

	// Auto-select the first server when servers load and nothing is selected.
	useEffect(() => {
		if (!selectedId && servers.length > 0) {
			setSelectedId(servers[0]?.id ?? null)
		}
		// If the selected server got deleted, clear the selection.
		if (selectedId && !servers.some((s) => s.id === selectedId)) {
			setSelectedId(servers[0]?.id ?? null)
		}
	}, [servers, selectedId])

	const selected = servers.find((s) => s.id === selectedId) ?? null

	return (
		<div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
			<Sidebar
				servers={servers}
				selectedId={selectedId}
				onSelect={setSelectedId}
				onAdd={() => setAddOpen(true)}
				onOpenSettings={() => {
					/* settings drawer is a follow-up PR */
				}}
			/>

			<section className="flex-1 overflow-hidden">
				{isLoading ? (
					<LoadingView />
				) : servers.length === 0 ? (
					<EmptyState onAdd={() => setAddOpen(true)} />
				) : selected ? (
					<ServerView key={selected.id} server={selected} onDeleted={() => setSelectedId(null)} />
				) : (
					<NoSelection />
				)}
			</section>

			<ServerFormDialog open={addOpen} onOpenChange={setAddOpen} />
		</div>
	)
}

function LoadingView() {
	return (
		<div className="grid h-full place-items-center text-sm text-muted-foreground">Loading…</div>
	)
}

function NoSelection() {
	return (
		<div className="grid h-full place-items-center p-12">
			<Empty className="max-w-md">
				<EmptyHeader>
					<EmptyTitle>No server selected</EmptyTitle>
					<EmptyDescription>Pick one from the sidebar to view its details.</EmptyDescription>
				</EmptyHeader>
			</Empty>
		</div>
	)
}

export default function App() {
	return (
		<Providers>
			<AppShell />
		</Providers>
	)
}
