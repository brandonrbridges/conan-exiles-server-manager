import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Textarea } from '@/components/ui/textarea'
import { useBroadcast } from '@/hooks/use-live-admin'
import { Megaphone, Send } from 'lucide-react'
import { useState } from 'react'

interface BroadcastComposerProps {
	serverId: string
}

const MAX_LEN = 500

export function BroadcastComposer({ serverId }: BroadcastComposerProps) {
	const [message, setMessage] = useState('')
	const broadcast = useBroadcast(serverId)

	const trimmed = message.trim()
	const canSend = trimmed.length > 0 && trimmed.length <= MAX_LEN && !broadcast.isPending

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault()
		if (!canSend) return
		broadcast.mutate(trimmed, {
			onSuccess: () => setMessage(''),
		})
	}

	return (
		<Card>
			<CardHeader className="pb-3">
				<CardTitle className="flex items-center gap-2 text-base">
					<Megaphone className="size-4 text-muted-foreground" aria-hidden="true" />
					Broadcast
				</CardTitle>
			</CardHeader>
			<CardContent>
				<form onSubmit={handleSubmit} className="flex flex-col gap-3">
					<Textarea
						placeholder="Server-wide chat message…"
						value={message}
						onChange={(e) => setMessage(e.target.value)}
						rows={2}
						maxLength={MAX_LEN}
					/>
					<div className="flex items-center justify-between">
						<span className="text-xs text-muted-foreground">
							{message.length}/{MAX_LEN} chars
						</span>
						<Button type="submit" size="sm" disabled={!canSend}>
							<Send className="size-4" />
							{broadcast.isPending ? 'Sending…' : 'Send'}
						</Button>
					</div>
				</form>
			</CardContent>
		</Card>
	)
}
