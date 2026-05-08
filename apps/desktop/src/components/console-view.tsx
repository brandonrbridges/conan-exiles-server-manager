import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { commands, formatError } from '@/lib/commands'
import { cn } from '@/lib/utils'
import { Send, Terminal } from 'lucide-react'
import { type FormEvent, type KeyboardEvent, useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'

const MAX_HISTORY = 200

interface ConsoleEntry {
	id: number
	command: string
	response: string
	at: number
	error?: string
}

interface ConsoleViewProps {
	serverId: string
}

/**
 * Power-user RCON console.
 *
 * Free-text input with ↑/↓ history navigation, in-memory scrollback
 * (capped at 200 entries), and per-entry expand/collapse via Tailwind's
 * `whitespace-pre-wrap`. State is per-server and lives only as long as
 * the view is mounted — persistence is a v1 concern.
 */
export function ConsoleView({ serverId }: ConsoleViewProps) {
	const [draft, setDraft] = useState('')
	const [entries, setEntries] = useState<ConsoleEntry[]>([])
	const [pending, setPending] = useState(false)
	const inputRef = useRef<HTMLInputElement>(null)
	const scrollAnchor = useRef<HTMLDivElement>(null)
	const historyCursor = useRef<number | null>(null)

	useEffect(() => {
		// Re-run on every append; we read the length only to drive the
		// dependency, the scroll itself doesn't need it.
		void entries.length
		scrollAnchor.current?.scrollIntoView({ behavior: 'smooth', block: 'end' })
	}, [entries.length])

	const handleSubmit = async (e: FormEvent) => {
		e.preventDefault()
		const cmd = draft.trim()
		if (!cmd || pending) return
		setPending(true)
		const id = Date.now()
		try {
			const response = await commands.sendCommand(serverId, cmd)
			append({ id, command: cmd, response, at: Date.now() })
		} catch (err) {
			const errMsg = formatError(err)
			append({ id, command: cmd, response: '', at: Date.now(), error: errMsg })
			toast.error(errMsg)
		} finally {
			setPending(false)
			setDraft('')
			historyCursor.current = null
			inputRef.current?.focus()
		}
	}

	const append = (entry: ConsoleEntry) => {
		setEntries((prev) => {
			const next = [...prev, entry]
			return next.length > MAX_HISTORY ? next.slice(-MAX_HISTORY) : next
		})
	}

	const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
		if (entries.length === 0) return
		if (e.key === 'ArrowUp') {
			e.preventDefault()
			const cur = historyCursor.current
			const nextIdx = cur === null ? entries.length - 1 : Math.max(0, cur - 1)
			historyCursor.current = nextIdx
			setDraft(entries[nextIdx]?.command ?? '')
		} else if (e.key === 'ArrowDown') {
			e.preventDefault()
			const cur = historyCursor.current
			if (cur === null) return
			const nextIdx = cur + 1
			if (nextIdx >= entries.length) {
				historyCursor.current = null
				setDraft('')
			} else {
				historyCursor.current = nextIdx
				setDraft(entries[nextIdx]?.command ?? '')
			}
		}
	}

	return (
		<div className="flex h-full flex-col gap-3">
			<div className="flex items-center gap-2">
				<Terminal className="size-4 text-muted-foreground" aria-hidden="true" />
				<h2 className="text-sm font-medium">RCON console</h2>
				<Badge variant="secondary" className="ml-auto font-mono text-xs">
					{entries.length}/{MAX_HISTORY}
				</Badge>
			</div>

			<ScrollArea className="flex-1 rounded-md border border-border bg-card">
				<div className="flex flex-col gap-3 p-4 font-mono text-xs">
					{entries.length === 0 ? (
						<EmptyHint />
					) : (
						entries.map((entry) => <ConsoleEntryRow key={entry.id} entry={entry} />)
					)}
					<div ref={scrollAnchor} />
				</div>
			</ScrollArea>

			<form onSubmit={handleSubmit} className="flex items-center gap-2">
				<span className="font-mono text-sm text-muted-foreground">$</span>
				<Input
					ref={inputRef}
					value={draft}
					onChange={(e) => {
						setDraft(e.target.value)
						historyCursor.current = null
					}}
					onKeyDown={handleKeyDown}
					placeholder='Type an RCON command. Try "help".'
					autoComplete="off"
					spellCheck={false}
					className="font-mono"
					disabled={pending}
				/>
				<Button type="submit" size="sm" disabled={!draft.trim() || pending}>
					<Send className="size-4" />
					{pending ? 'Running…' : 'Run'}
				</Button>
			</form>
		</div>
	)
}

function ConsoleEntryRow({ entry }: { entry: ConsoleEntry }) {
	const time = new Date(entry.at).toLocaleTimeString()
	return (
		<div className="flex flex-col gap-1">
			<div className="flex items-baseline gap-2 text-muted-foreground">
				<span className="text-[10px]">{time}</span>
				<span className="text-foreground">$ {entry.command}</span>
			</div>
			<pre
				className={cn(
					'whitespace-pre-wrap pl-4 leading-relaxed',
					entry.error ? 'text-destructive' : 'text-muted-foreground',
				)}
			>
				{entry.error ? entry.error : entry.response || <span className="italic">(no output)</span>}
			</pre>
		</div>
	)
}

function EmptyHint() {
	return (
		<div className="flex flex-col gap-1 text-muted-foreground">
			<p className="not-italic">Power-user escape hatch — sends raw RCON commands.</p>
			<p>
				Use <span className="text-foreground">↑/↓</span> to navigate command history. Try{' '}
				<span className="text-foreground">help</span> to see what the server supports.
			</p>
		</div>
	)
}
