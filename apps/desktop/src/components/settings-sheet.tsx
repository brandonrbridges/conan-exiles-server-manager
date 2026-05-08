import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import {
	Sheet,
	SheetContent,
	SheetDescription,
	SheetHeader,
	SheetTitle,
} from '@/components/ui/sheet'
import { type Theme, useTheme } from '@/lib/theme'
import { Monitor, Moon, Sun } from 'lucide-react'

const THEME_OPTIONS: ReadonlyArray<{
	value: Theme
	label: string
	description: string
	icon: typeof Sun
}> = [
	{
		value: 'system',
		label: 'System',
		description: "Follow your operating system's appearance.",
		icon: Monitor,
	},
	{
		value: 'light',
		label: 'Light',
		description: 'Always use the light theme.',
		icon: Sun,
	},
	{
		value: 'dark',
		label: 'Dark',
		description: 'Always use the dark theme.',
		icon: Moon,
	},
]

interface SettingsSheetProps {
	open: boolean
	onOpenChange: (open: boolean) => void
}

export function SettingsSheet({ open, onOpenChange }: SettingsSheetProps) {
	const { theme, setTheme } = useTheme()

	return (
		<Sheet open={open} onOpenChange={onOpenChange}>
			<SheetContent side="right" className="w-96 sm:max-w-sm">
				<SheetHeader>
					<SheetTitle>Settings</SheetTitle>
					<SheetDescription>
						App-level preferences. Per-server settings live on each server's view.
					</SheetDescription>
				</SheetHeader>

				<div className="flex flex-col gap-6 px-4 pb-6">
					<section className="flex flex-col gap-3">
						<div className="flex flex-col gap-1">
							<h3 className="text-sm font-medium">Appearance</h3>
							<p className="text-xs text-muted-foreground">Choose how the app looks.</p>
						</div>

						<RadioGroup
							value={theme}
							onValueChange={(value) => setTheme(value as Theme)}
							className="flex flex-col gap-2"
						>
							{THEME_OPTIONS.map((opt) => {
								const Icon = opt.icon
								return (
									<Label
										key={opt.value}
										htmlFor={`theme-${opt.value}`}
										className="flex cursor-pointer items-start gap-3 rounded-md border border-border p-3 transition-colors hover:bg-accent hover:text-accent-foreground has-[:checked]:border-primary has-[:checked]:bg-accent"
									>
										<RadioGroupItem id={`theme-${opt.value}`} value={opt.value} />
										<Icon className="size-4 mt-0.5 text-muted-foreground" aria-hidden="true" />
										<div className="flex flex-col gap-0.5">
											<span className="text-sm font-medium leading-none">{opt.label}</span>
											<span className="text-xs text-muted-foreground">{opt.description}</span>
										</div>
									</Label>
								)
							})}
						</RadioGroup>
					</section>
				</div>
			</SheetContent>
		</Sheet>
	)
}
