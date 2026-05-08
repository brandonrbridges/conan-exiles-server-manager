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
import { useBans, useUnbanPlayer } from '@/hooks/use-live-admin'
import { ShieldOff } from 'lucide-react'

interface BansTableProps {
	serverId: string
}

const SKELETON_KEYS = ['b1', 'b2', 'b3'] as const

export function BansTable({ serverId }: BansTableProps) {
	const { data: bans, isLoading, isError } = useBans(serverId)
	const unban = useUnbanPlayer(serverId)

	if (isLoading) return <BansTableSkeleton />
	if (isError) return <BansTableError />
	if (!bans || bans.length === 0) return <BansTableEmpty />

	return (
		<Table>
			<TableHeader>
				<TableRow>
					<TableHead>FuncomID</TableHead>
					<TableHead>Platform ID</TableHead>
					<TableHead>Player name</TableHead>
					<TableHead>Reason</TableHead>
					<TableHead className="text-right">Actions</TableHead>
				</TableRow>
			</TableHeader>
			<TableBody>
				{bans.map((ban) => (
					<TableRow key={`${ban.user_id}-${ban.platform_id}`}>
						<TableCell className="font-mono text-xs">{ban.user_id}</TableCell>
						<TableCell className="font-mono text-xs text-muted-foreground">
							{ban.platform_id}
						</TableCell>
						<TableCell>{ban.player_name ?? '—'}</TableCell>
						<TableCell className="max-w-xs truncate text-muted-foreground">
							{ban.reason ?? '—'}
						</TableCell>
						<TableCell className="text-right">
							<Button
								variant="outline"
								size="sm"
								disabled={unban.isPending}
								onClick={() => unban.mutate(ban.user_id)}
							>
								<ShieldOff className="size-4" />
								Unban
							</Button>
						</TableCell>
					</TableRow>
				))}
			</TableBody>
		</Table>
	)
}

function BansTableSkeleton() {
	return (
		<div className="flex flex-col gap-2">
			{SKELETON_KEYS.map((k) => (
				<Skeleton key={k} className="h-12 rounded-md" />
			))}
		</div>
	)
}

function BansTableEmpty() {
	return (
		<div className="grid h-48 place-items-center">
			<Empty className="border-dashed">
				<EmptyHeader>
					<EmptyMedia variant="icon">
						<ShieldOff className="size-6" aria-hidden="true" />
					</EmptyMedia>
					<EmptyTitle>No active bans</EmptyTitle>
					<EmptyDescription>
						Banned players appear here. Refreshes every minute while this view is open.
					</EmptyDescription>
				</EmptyHeader>
			</Empty>
		</div>
	)
}

function BansTableError() {
	return (
		<div className="rounded-md border border-destructive/40 bg-destructive/10 p-4 text-sm text-destructive-foreground">
			Couldn't load the bans list. The connection may be reconnecting — it'll retry on the next
			polling tick.
		</div>
	)
}
