import { Sword } from 'lucide-react'

export default function App() {
	return (
		<main className="grid min-h-screen place-items-center bg-background p-8 text-foreground">
			<section className="flex max-w-xl flex-col items-center gap-6 text-center">
				<div className="flex size-16 items-center justify-center rounded-2xl bg-primary text-primary-foreground shadow-lg">
					<Sword className="size-8" aria-hidden="true" />
				</div>
				<div className="flex flex-col gap-2">
					<h1 className="text-3xl font-semibold tracking-tight">
						Conan Exiles | Server Manager Enhanced
					</h1>
					<p className="text-sm text-muted-foreground">
						Pre-alpha. Scaffold only — RCON, persistence, and live admin land in upcoming PRs.
					</p>
				</div>
			</section>
		</main>
	)
}
