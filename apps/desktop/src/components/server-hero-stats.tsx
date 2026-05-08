import { Skeleton } from '@/components/ui/skeleton'
import { useServerOverview } from '@/hooks/use-live-admin'
import type { ServerOverview, ServerSetting } from '@/types/generated'

interface ServerHeroStatsProps {
	serverId: string
}

/**
 * Top-of-dashboard strip: player count + a curated row of runtime settings
 * pulled from `GetServerSetting`. Replaces the placeholder summary card.
 *
 * Only the verified-working settings keys are shown — see the Rust
 * `OVERVIEW_KEYS` list. Keys the server doesn't expose are omitted, not
 * rendered as "—" / "missing", to keep the strip clean.
 */
export function ServerHeroStats({ serverId }: ServerHeroStatsProps) {
	const { data, isLoading } = useServerOverview(serverId)

	if (isLoading) return <HeroStatsSkeleton />
	if (!data) return null

	const visible = visibleSettings(data)
	if (visible.length === 0) return <PlayerCountOnly count={data.player_count} />

	return (
		<div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
			<StatCard label="Players online" value={`${data.player_count}`} />
			{visible.map((setting) => (
				<StatCard key={setting.key} label={prettyKey(setting.key)} value={prettyValue(setting)} />
			))}
		</div>
	)
}

function PlayerCountOnly({ count }: { count: number }) {
	return (
		<div className="grid grid-cols-1">
			<StatCard label="Players online" value={`${count}`} />
		</div>
	)
}

function StatCard({ label, value }: { label: string; value: string }) {
	return (
		<div className="flex flex-col gap-1 rounded-md border border-border bg-card px-4 py-3">
			<span className="text-xs uppercase tracking-wide text-muted-foreground">{label}</span>
			<span className="text-sm font-semibold">{value}</span>
		</div>
	)
}

const HERO_SKELETON_KEYS = ['h1', 'h2', 'h3', 'h4', 'h5'] as const

function HeroStatsSkeleton() {
	return (
		<div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
			{HERO_SKELETON_KEYS.map((k) => (
				<Skeleton key={k} className="h-16 rounded-md" />
			))}
		</div>
	)
}

function visibleSettings(overview: ServerOverview): ServerSetting[] {
	// Filter: skip empty values and AdminPassword (already shown elsewhere
	// and shouldn't be on display anyway, even if Conan returns it).
	return overview.settings.filter(
		(s) => s.value.trim() !== '' && s.key.toLowerCase() !== 'adminpassword',
	)
}

const KEY_LABELS: Record<string, string> = {
	PVPEnabled: 'PVP',
	ServerRegion: 'Region',
	HarvestAmountMultiplier: 'Harvest',
	ItemSpoilRateScale: 'Spoil rate',
	ResourceRespawnSpeedMultiplier: 'Respawn',
	StaminaCostMultiplier: 'Stamina',
	ClanMaxSize: 'Clan max',
	ChatMaxMessageLength: 'Chat max',
	IsBattlEyeEnabled: 'BattlEye',
}

function prettyKey(key: string): string {
	return KEY_LABELS[key] ?? key
}

function prettyValue({ key, value }: ServerSetting): string {
	// Booleans Conan returns as "true"/"false" — Title-case for display.
	if (value === 'true' || value === 'false') {
		return value === 'true' ? 'On' : 'Off'
	}
	if (key.toLowerCase() === 'isbattleyeenabled') {
		return value === 'true' ? 'On' : 'Off'
	}
	return value
}
