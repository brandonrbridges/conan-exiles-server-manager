import { commands, formatError } from '@/lib/commands'
import type { Server, ServerInput, TestConnectionInput } from '@/types/generated'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

const SERVERS_KEY = ['servers'] as const

export function useServers() {
	return useQuery({
		queryKey: SERVERS_KEY,
		queryFn: () => commands.listServers(),
	})
}

export function useSaveServer() {
	const qc = useQueryClient()
	return useMutation({
		mutationFn: (input: ServerInput) => commands.saveServer(input),
		onSuccess: (_saved: Server) => {
			qc.invalidateQueries({ queryKey: SERVERS_KEY })
			toast.success('Server saved')
		},
		onError: (err) => {
			toast.error(formatError(err))
		},
	})
}

export function useDeleteServer() {
	const qc = useQueryClient()
	return useMutation({
		mutationFn: (id: string) => commands.deleteServer(id),
		onSuccess: () => {
			qc.invalidateQueries({ queryKey: SERVERS_KEY })
			toast.success('Server deleted')
		},
		onError: (err) => {
			toast.error(formatError(err))
		},
	})
}

export function useTestConnection() {
	return useMutation({
		mutationFn: (input: TestConnectionInput) => commands.testConnection(input),
	})
}
