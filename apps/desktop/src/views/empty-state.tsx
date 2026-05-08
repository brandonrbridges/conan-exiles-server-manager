import { Button } from '@/components/ui/button'
import {
	Empty,
	EmptyContent,
	EmptyDescription,
	EmptyHeader,
	EmptyMedia,
	EmptyTitle,
} from '@/components/ui/empty'
import { Plus, Server } from 'lucide-react'

interface EmptyStateProps {
	onAdd: () => void
}

export function EmptyState({ onAdd }: EmptyStateProps) {
	return (
		<div className="grid h-full place-items-center p-12">
			<Empty className="max-w-md">
				<EmptyHeader>
					<EmptyMedia variant="icon">
						<Server className="size-6" aria-hidden="true" />
					</EmptyMedia>
					<EmptyTitle>No servers yet</EmptyTitle>
					<EmptyDescription>
						Add your first Conan Exiles Enhanced dedicated server to manage it from here.
						Credentials live in your OS keychain — they never leave your machine.
					</EmptyDescription>
				</EmptyHeader>
				<EmptyContent>
					<Button onClick={onAdd}>
						<Plus className="size-4" />
						Add your first server
					</Button>
				</EmptyContent>
			</Empty>
		</div>
	)
}
