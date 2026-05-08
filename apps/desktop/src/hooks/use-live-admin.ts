import { commands, formatError } from '@/lib/commands'
import type { BanInput, KickInput } from '@/types/generated'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

const PLAYERS_POLL_MS = 10_000
const OVERVIEW_POLL_MS = 30_000
const BANS_POLL_MS = 60_000

/**
 * Polls `listplayers` while mounted. Default cadence per the v0 design
 * (10s); the polling interval setting will plumb through here when we
 * wire it up.
 */
export function usePlayers(serverId: string | null) {
	return useQuery({
		queryKey: ['players', serverId] as const,
		queryFn: () => {
			if (!serverId) throw new Error('serverId required')
			return commands.listPlayers(serverId)
		},
		enabled: !!serverId,
		refetchInterval: PLAYERS_POLL_MS,
		refetchIntervalInBackground: false,
	})
}

/**
 * Polls `listbans`. Slower than the player list because bans change far
 * less often.
 */
export function useBans(serverId: string | null) {
	return useQuery({
		queryKey: ['bans', serverId] as const,
		queryFn: () => {
			if (!serverId) throw new Error('serverId required')
			return commands.listBans(serverId)
		},
		enabled: !!serverId,
		refetchInterval: BANS_POLL_MS,
		refetchIntervalInBackground: false,
	})
}

/**
 * Aggregated server hero stats — player count + a verified bundle of
 * `GetServerSetting` values. Polls slowly because runtime settings change
 * rarely; player count gets fresher coverage from `usePlayers`.
 */
export function useServerOverview(serverId: string | null) {
	return useQuery({
		queryKey: ['serverOverview', serverId] as const,
		queryFn: () => {
			if (!serverId) throw new Error('serverId required')
			return commands.serverOverview(serverId)
		},
		enabled: !!serverId,
		refetchInterval: OVERVIEW_POLL_MS,
		refetchIntervalInBackground: false,
	})
}

export function useBroadcast(serverId: string | null) {
	const qc = useQueryClient()
	return useMutation({
		mutationFn: (message: string) => {
			if (!serverId) throw new Error('serverId required')
			return commands.broadcast(serverId, message)
		},
		onSuccess: () => {
			toast.success('Message broadcast.')
			qc.invalidateQueries({ queryKey: ['players', serverId] })
		},
		onError: (err) => {
			toast.error(formatError(err))
		},
	})
}

export function useKickPlayer(serverId: string | null) {
	const qc = useQueryClient()
	return useMutation({
		mutationFn: (input: KickInput) => {
			if (!serverId) throw new Error('serverId required')
			return commands.kickPlayer(serverId, input)
		},
		onSuccess: (response) => {
			toast.success(response.trim() || 'Kick sent.')
			qc.invalidateQueries({ queryKey: ['players', serverId] })
		},
		onError: (err) => {
			toast.error(formatError(err))
		},
	})
}

export function useBanPlayer(serverId: string | null) {
	const qc = useQueryClient()
	return useMutation({
		mutationFn: (input: BanInput) => {
			if (!serverId) throw new Error('serverId required')
			return commands.banPlayer(serverId, input)
		},
		onSuccess: (response) => {
			toast.success(response.trim() || 'Ban sent.')
			qc.invalidateQueries({ queryKey: ['players', serverId] })
			qc.invalidateQueries({ queryKey: ['bans', serverId] })
		},
		onError: (err) => {
			toast.error(formatError(err))
		},
	})
}

export function useUnbanPlayer(serverId: string | null) {
	const qc = useQueryClient()
	return useMutation({
		mutationFn: (userOrPlatformId: string) => {
			if (!serverId) throw new Error('serverId required')
			return commands.unbanPlayer(serverId, userOrPlatformId)
		},
		onSuccess: (response) => {
			toast.success(response.trim() || 'Unbanned.')
			qc.invalidateQueries({ queryKey: ['bans', serverId] })
		},
		onError: (err) => {
			toast.error(formatError(err))
		},
	})
}
