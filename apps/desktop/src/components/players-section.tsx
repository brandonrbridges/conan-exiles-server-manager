import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { BansTable } from './bans-table'
import { PlayerTable } from './player-table'

interface PlayersSectionProps {
	serverId: string
}

/**
 * Combined Players + Bans area for the dashboard. Uses sub-tabs to flip
 * between "online now" and "currently banned", matching the v0 design's
 * sub-tab spec.
 */
export function PlayersSection({ serverId }: PlayersSectionProps) {
	return (
		<Tabs defaultValue="online" className="flex flex-col gap-3">
			<TabsList>
				<TabsTrigger value="online">Online</TabsTrigger>
				<TabsTrigger value="bans">Bans</TabsTrigger>
			</TabsList>
			<TabsContent value="online" className="m-0">
				<PlayerTable serverId={serverId} />
			</TabsContent>
			<TabsContent value="bans" className="m-0">
				<BansTable serverId={serverId} />
			</TabsContent>
		</Tabs>
	)
}
