import { commands, formatError } from '@/lib/commands'
import type { ConnectionState } from '@/types/generated'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useEffect } from 'react'
import { toast } from 'sonner'

const POLL_INTERVAL_MS = 1_000

/**
 * Open the RCON connection for `serverId` while this hook is mounted, polls
 * its state every second, and closes it on unmount.
 *
 * Used by the per-server view. Selecting a different server in the sidebar
 * unmounts this hook with the old id and remounts with the new one — clean
 * lifecycle.
 */
export function useActiveConnection(serverId: string | null) {
	const qc = useQueryClient()

	useEffect(() => {
		if (!serverId) return

		let cancelled = false
		commands.openConnection(serverId).catch((err) => {
			if (!cancelled) toast.error(formatError(err))
		})

		return () => {
			cancelled = true
			commands.closeConnection(serverId).catch(() => {
				// Best-effort cleanup; if the registry already dropped the
				// handle (e.g. on app shutdown) the error is uninteresting.
			})
			qc.removeQueries({ queryKey: ['connectionState', serverId] })
		}
	}, [serverId, qc])

	return useQuery<ConnectionState>({
		queryKey: ['connectionState', serverId],
		queryFn: () => {
			if (!serverId) throw new Error('serverId is required')
			return commands.connectionState(serverId)
		},
		enabled: !!serverId,
		refetchInterval: POLL_INTERVAL_MS,
		refetchIntervalInBackground: false,
	})
}
