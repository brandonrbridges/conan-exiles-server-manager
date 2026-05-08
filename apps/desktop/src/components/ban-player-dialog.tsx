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
import { useBanPlayer } from '@/hooks/use-live-admin'
import type { Player } from '@/types/generated'
import { useEffect, useState } from 'react'

interface BanPlayerDialogProps {
	serverId: string
	target: Player | null
	onOpenChange: (open: boolean) => void
}

export function BanPlayerDialog({ serverId, target, onOpenChange }: BanPlayerDialogProps) {
	const ban = useBanPlayer(serverId)
	const [reason, setReason] = useState('')

	useEffect(() => {
		if (target) setReason('')
	}, [target])

	if (!target) return null

	const handleConfirm = () => {
		ban.mutate(
			{
				kind: 'name',
				identifier: target.char_name,
				message: reason.trim() || 'Banned by an admin.',
			},
			{ onSuccess: () => onOpenChange(false) },
		)
	}

	return (
		<Dialog open={!!target} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Ban {target.char_name}?</DialogTitle>
					<DialogDescription>
						Bans this player by their FuncomID. They won't be able to rejoin until you unban them
						from the Bans tab.
					</DialogDescription>
				</DialogHeader>

				<div className="flex flex-col gap-2">
					<Label htmlFor="ban-reason">Reason (shown to the player)</Label>
					<Textarea
						id="ban-reason"
						placeholder="Optional — a default is sent if blank."
						value={reason}
						onChange={(e) => setReason(e.target.value)}
						rows={3}
					/>
				</div>

				<DialogFooter>
					<Button variant="ghost" onClick={() => onOpenChange(false)} disabled={ban.isPending}>
						Cancel
					</Button>
					<Button
						onClick={handleConfirm}
						disabled={ban.isPending}
						className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
					>
						{ban.isPending ? 'Banning…' : 'Ban'}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	)
}
