import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { Server } from '@/types/generated'
import { Plus, Settings, Sword } from 'lucide-react'
import { ServerSidebarItem } from './server-sidebar-item'

interface SidebarProps {
	servers: Server[]
	selectedId: string | null
	onSelect: (id: string) => void
	onAdd: () => void
	onOpenSettings: () => void
}

export function Sidebar({ servers, selectedId, onSelect, onAdd, onOpenSettings }: SidebarProps) {
	return (
		<aside className="flex h-screen w-64 shrink-0 flex-col border-r border-border bg-card">
			<header className="flex items-center gap-3 px-4 py-4">
				<div className="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
					<Sword className="size-4" aria-hidden="true" />
				</div>
				<div className="flex flex-col leading-tight">
					<span className="text-sm font-semibold">CESM</span>
					<span className="text-xs text-muted-foreground">Pre-alpha</span>
				</div>
			</header>

			<Separator />

			<div className="flex items-center justify-between px-4 py-3">
				<span className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
					Servers
				</span>
				<Tooltip>
					<TooltipTrigger asChild>
						<Button
							variant="ghost"
							size="icon"
							className="size-7"
							onClick={onAdd}
							aria-label="Add server"
						>
							<Plus className="size-4" />
						</Button>
					</TooltipTrigger>
					<TooltipContent side="right">Add server</TooltipContent>
				</Tooltip>
			</div>

			<nav className={cn('flex-1 overflow-y-auto px-2', servers.length === 0 && 'opacity-50')}>
				{servers.length === 0 ? (
					<p className="px-2 text-xs text-muted-foreground">
						No servers yet. Click <span className="font-medium">+</span> to add one.
					</p>
				) : (
					<ul className="flex flex-col gap-1">
						{servers.map((server) => (
							<ServerSidebarItem
								key={server.id}
								server={server}
								selected={server.id === selectedId}
								onSelect={() => onSelect(server.id)}
							/>
						))}
					</ul>
				)}
			</nav>

			<Separator />

			<footer className="flex justify-end px-4 py-3">
				<Tooltip>
					<TooltipTrigger asChild>
						<Button
							variant="ghost"
							size="icon"
							className="size-8"
							onClick={onOpenSettings}
							aria-label="Settings"
						>
							<Settings className="size-4" />
						</Button>
					</TooltipTrigger>
					<TooltipContent side="right">Settings</TooltipContent>
				</Tooltip>
			</footer>
		</aside>
	)
}
