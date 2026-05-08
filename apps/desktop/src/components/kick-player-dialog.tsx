import { Button } from '@/components/ui/button'
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { useKickPlayer } from '@/hooks/use-live-admin'
import type { Player } from '@/types/generated'
import { useEffect, useState } from 'react'

interface KickPlayerDialogProps {
	serverId: string
	target: Player | null
	onOpenChange: (open: boolean) => void
}

export function KickPlayerDialog({ serverId, target, onOpenChange }: KickPlayerDialogProps) {
	const kick = useKickPlayer(serverId)
	const [reason, setReason] = useState('')

	useEffect(() => {
		if (target) setReason('')
	}, [target])

	if (!target) return null

	const handleConfirm = () => {
		kick.mutate(
			{
				kind: 'name',
				identifier: target.char_name,
				message: reason.trim() || 'Kicked by an admin.',
			},
			{ onSuccess: () => onOpenChange(false) },
		)
	}

	return (
		<Dialog open={!!target} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Kick {target.char_name}?</DialogTitle>
					<DialogDescription>
						Disconnects the player from the server. They can rejoin straight away.
					</DialogDescription>
				</DialogHeader>

				<div className="flex flex-col gap-2">
					<Label htmlFor="kick-reason">Reason (shown to the player)</Label>
					<Textarea
						id="kick-reason"
						placeholder="Optional — a default is sent if blank."
						value={reason}
						onChange={(e) => setReason(e.target.value)}
						rows={3}
					/>
				</div>

				<DialogFooter>
					<Button variant="ghost" onClick={() => onOpenChange(false)} disabled={kick.isPending}>
						Cancel
					</Button>
					<Button onClick={handleConfirm} disabled={kick.isPending}>
						{kick.isPending ? 'Kicking…' : 'Kick'}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
