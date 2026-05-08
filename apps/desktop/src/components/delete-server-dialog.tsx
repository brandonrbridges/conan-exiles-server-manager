import {
	AlertDialog,
	AlertDialogAction,
	AlertDialogCancel,
	AlertDialogContent,
	AlertDialogDescription,
	AlertDialogFooter,
	AlertDialogHeader,
	AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { useDeleteServer } from '@/hooks/use-servers'
import type { Server } from '@/types/generated'

interface DeleteServerDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	server: Server | null
	onDeleted?: () => void
}

export function DeleteServerDialog({
	open,
	onOpenChange,
	server,
	onDeleted,
}: DeleteServerDialogProps) {
	const remove = useDeleteServer()

	const handleConfirm = () => {
		if (!server) return
		remove.mutate(server.id, {
			onSuccess: () => {
				onOpenChange(false)
				onDeleted?.()
			},
		})
	}

	return (
		<AlertDialog open={open} onOpenChange={onOpenChange}>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Delete server?</AlertDialogTitle>
					<AlertDialogDescription>
						{server ? (
							<>
								Removes <span className="font-medium">{server.name}</span> from this app and wipes
								its keychain entries. The server itself is unaffected. This can't be undone.
							</>
						) : null}
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel disabled={remove.isPending}>Cancel</AlertDialogCancel>
					<AlertDialogAction
						onClick={handleConfirm}
						disabled={remove.isPending}
						className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
					>
						{remove.isPending ? 'Deleting…' : 'Delete'}
					</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	)
}
