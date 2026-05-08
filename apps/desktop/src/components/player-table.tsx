import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from '@/components/ui/empty'
import { Skeleton } from '@/components/ui/skeleton'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '@/components/ui/table'
import { usePlayers } from '@/hooks/use-live-admin'
import type { Player } from '@/types/generated'
import { Ban, LogOut, Users } from 'lucide-react'
import { useState } from 'react'
import { BanPlayerDialog } from './ban-player-dialog'
import { KickPlayerDialog } from './kick-player-dialog'

interface PlayerTableProps {
	serverId: string
}

export function PlayerTable({ serverId }: PlayerTableProps) {
	const { data: players, isLoading, isError } = usePlayers(serverId)
	const [kickTarget, setKickTarget] = useState<Player | null>(null)
	const [banTarget, setBanTarget] = useState<Player | null>(null)

	if (isLoading) return <PlayerTableSkeleton />
	if (isError) return <PlayerTableError />
	if (!players || players.length === 0) return <PlayerTableEmpty />

	return (
		<>
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead className="w-12">#</TableHead>
						<TableHead>Character</TableHead>
						<TableHead>Player</TableHead>
						<TableHead>Platform</TableHead>
						<TableHead className="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{players.map((player) => (
						<TableRow key={`${player.idx}-${player.user_id}`}>
							<TableCell className="font-mono text-xs text-muted-foreground">
								{player.idx}
							</TableCell>
							<TableCell className="font-medium">{player.char_name}</TableCell>
							<TableCell>
								<div className="flex flex-col gap-0.5">
									<span>{player.player_name}</span>
									<span className="text-xs text-muted-foreground">ID {player.user_id}</span>
								</div>
							</TableCell>
							<TableCell>
								<div className="flex flex-col gap-0.5">
									<Badge variant="secondary">{player.platform_name}</Badge>
									<span className="text-xs text-muted-foreground font-mono">
										{player.platform_id}
									</span>
								</div>
							</TableCell>
							<TableCell className="text-right">
								<div className="flex justify-end gap-2">
									<Button variant="outline" size="sm" onClick={() => setKickTarget(player)}>
										<LogOut className="size-4" />
										Kick
									</Button>
									<Button variant="outline" size="sm" onClick={() => setBanTarget(player)}>
										<Ban className="size-4" />
										Ban
									</Button>
								</div>
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>

			<KickPlayerDialog
				serverId={serverId}
				target={kickTarget}
				onOpenChange={(open) => !open && setKickTarget(null)}
			/>
			<BanPlayerDialog
				serverId={serverId}
				target={banTarget}
				onOpenChange={(open) => !open && setBanTarget(null)}
			/>
		</>
	)
}

const SKELETON_KEYS = ['s1', 's2', 's3'] as const

function PlayerTableSkeleton() {
	return (
		<div className="flex flex-col gap-2">
			{SKELETON_KEYS.map((k) => (
				<Skeleton key={k} className="h-12 rounded-md" />
			))}
		</div>
	)
}

function PlayerTableEmpty() {
	return (
		<div className="grid h-48 place-items-center">
			<Empty className="border-dashed">
				<EmptyHeader>
					<EmptyMedia variant="icon">
						<Users className="size-6" aria-hidden="true" />
					</EmptyMedia>
					<EmptyTitle>No one's online</EmptyTitle>
					<EmptyDescription>
						The player list refreshes every 10 seconds while this view is open.
					</EmptyDescription>
				</EmptyHeader>
			</Empty>
		</div>
	)
}

function PlayerTableError() {
	return (
		<div className="rounded-md border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive-foreground">
			Couldn't load the player list. The connection may be reconnecting — it'll retry on the next
			polling tick.
		</div>
	)
}
