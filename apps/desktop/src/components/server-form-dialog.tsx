import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from '@/components/ui/dialog'
import type { Server } from '@/types/generated'
import { ServerForm } from './server-form'

interface ServerFormDialogProps {
	open: boolean
	onOpenChange: (open: boolean) => void
	server?: Server
}

export function ServerFormDialog({ open, onOpenChange, server }: ServerFormDialogProps) {
	const isEdit = !!server

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent className="max-w-md">
				<DialogHeader>
					<DialogTitle>{isEdit ? 'Edit server' : 'Add server'}</DialogTitle>
					<DialogDescription>
						{isEdit
							? 'Update the connection details. Passwords are blank — re-enter to save.'
							: 'Connect to your Conan Exiles Enhanced dedicated server over RCON.'}
					</DialogDescription>
				</DialogHeader>
				{server ? (
					<ServerForm server={server} onDone={() => onOpenChange(false)} />
				) : (
					<ServerForm onDone={() => onOpenChange(false)} />
				)}
			</DialogContent>
		</Dialog>
	)
}
