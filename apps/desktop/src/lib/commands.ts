import type {
	AppError,
	BanInput,
	BannedPlayer,
	ConnectionState,
	KickInput,
	Player,
	Server,
	ServerInput,
	ServerOverview,
	TestConnectionInput,
} from '@/types/generated'
import { invoke } from '@tauri-apps/api/core'

/**
 * Typed wrappers around every `#[tauri::command]` exposed by the Rust core.
 *
 * Promises reject with the serialised {@link AppError} discriminated union,
 * not a raw string. Callers `try/catch` and switch on `(err as AppError).kind`
 * for typed handling.
 */
export const commands = {
	// --- server CRUD + connection lifecycle ---
	listServers: () => invoke<Server[]>('list_servers'),
	saveServer: (input: ServerInput) => invoke<Server>('save_server', { input }),
	deleteServer: (id: string) => invoke<void>('delete_server', { id }),
	testConnection: (input: TestConnectionInput) => invoke<void>('test_connection', { input }),
	openConnection: (serverId: string) => invoke<void>('open_connection', { serverId }),
	closeConnection: (serverId: string) => invoke<void>('close_connection', { serverId }),
	connectionState: (serverId: string) => invoke<ConnectionState>('connection_state', { serverId }),
	sendCommand: (serverId: string, command: string) =>
		invoke<string>('send_command', { serverId, command }),

	// --- app settings ---
	getSetting: (key: string) => invoke<string | null>('get_setting', { key }),
	setSetting: (key: string, value: string) => invoke<void>('set_setting', { key, value }),

	// --- live admin (per-server) ---
	listPlayers: (serverId: string) => invoke<Player[]>('list_players', { serverId }),
	listBans: (serverId: string) => invoke<BannedPlayer[]>('list_bans', { serverId }),
	kickPlayer: (serverId: string, input: KickInput) =>
		invoke<string>('kick_player', { serverId, input }),
	banPlayer: (serverId: string, input: BanInput) =>
		invoke<string>('ban_player', { serverId, input }),
	unbanPlayer: (serverId: string, userOrPlatformId: string) =>
		invoke<string>('unban_player', { serverId, userOrPlatformId }),
	broadcast: (serverId: string, message: string) =>
		invoke<void>('broadcast', { serverId, message }),
	serverOverview: (serverId: string) => invoke<ServerOverview>('server_overview', { serverId }),
} as const

/**
 * Narrow an unknown caught value into an {@link AppError} when possible.
 * Returns `null` if the value doesn't match the discriminated-union shape.
 */
export function asAppError(err: unknown): AppError | null {
	if (typeof err === 'object' && err !== null && 'kind' in err) {
		return err as AppError
	}
	return null
}

/**
 * Format an unknown error for display in a toast or status line. Falls
 * back to a generic message rather than dumping a stack trace.
 */
export function formatError(err: unknown): string {
	const app = asAppError(err)
	if (app) {
		switch (app.kind) {
			case 'server_not_found':
				return 'Server not found.'
			case 'auth_failed':
				return 'Authentication failed. Check the RCON password.'
			case 'not_connected':
				return 'Not connected to the server.'
			case 'timeout':
				return 'The server took too long to respond.'
			case 'rate_limited':
				return 'Server is rate-limiting RCON requests. Try again in a moment.'
			case 'storage':
				return `Storage error: ${app.message}`
			case 'keychain':
				return `Keychain error: ${app.message}`
			case 'rcon':
				return `RCON error: ${app.message}`
			case 'invalid':
				return app.message
			case 'internal':
				return `Something went wrong: ${app.message}`
		}
	}
	if (err instanceof Error) return err.message
	return 'An unexpected error occurred.'
}
