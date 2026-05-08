import { Button } from '@/components/ui/button'
import {
	Form,
	FormControl,
	FormDescription,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { useSaveServer, useTestConnection } from '@/hooks/use-servers'
import { formatError } from '@/lib/commands'
import type { Server } from '@/types/generated'
import { zodResolver } from '@hookform/resolvers/zod'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { toast } from 'sonner'
import { z } from 'zod'

const schema = z.object({
	name: z.string().trim().min(1, 'Name is required'),
	host: z.string().trim().min(1, 'Host is required'),
	rcon_port: z
		.number({ message: 'Port must be a number' })
		.int('Port must be a whole number')
		.min(1, 'Port must be 1–65535')
		.max(65535, 'Port must be 1–65535'),
	rcon_password: z.string().min(1, 'RCON password is required'),
	admin_password: z.string().optional(),
})

export type ServerFormValues = z.infer<typeof schema>

interface ServerFormProps {
	/** When set, the form starts in edit mode. Passwords are intentionally
	 *  blank — they live in the keychain and have to be re-entered on save. */
	server?: Server
	onDone: () => void
}

export function ServerForm({ server, onDone }: ServerFormProps) {
	const save = useSaveServer()
	const test = useTestConnection()
	const [testStatus, setTestStatus] = useState<null | 'ok' | 'fail'>(null)

	const form = useForm<ServerFormValues>({
		resolver: zodResolver(schema),
		defaultValues: {
			name: server?.name ?? '',
			host: server?.host ?? '',
			rcon_port: server?.rcon_port ?? 7779,
			rcon_password: '',
			admin_password: '',
		},
	})

	const onSubmit = (values: ServerFormValues) => {
		save.mutate(
			{
				id: server?.id ?? null,
				name: values.name,
				host: values.host,
				rcon_port: values.rcon_port,
				rcon_password: values.rcon_password,
				admin_password: values.admin_password?.trim() ? values.admin_password : null,
			},
			{
				onSuccess: () => onDone(),
			},
		)
	}

	const onTest = async () => {
		setTestStatus(null)
		const valid = await form.trigger(['host', 'rcon_port', 'rcon_password'])
		if (!valid) return

		const v = form.getValues()
		try {
			await test.mutateAsync({
				host: v.host.trim(),
				rcon_port: v.rcon_port,
				rcon_password: v.rcon_password,
			})
			setTestStatus('ok')
			toast.success('Connection successful')
		} catch (err) {
			setTestStatus('fail')
			toast.error(formatError(err))
		}
	}

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-4">
				<FormField
					control={form.control}
					name="name"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Name</FormLabel>
							<FormControl>
								<Input placeholder="My Conan server" autoComplete="off" {...field} />
							</FormControl>
							<FormMessage />
						</FormItem>
					)}
				/>

				<div className="grid grid-cols-3 gap-3">
					<div className="col-span-2">
						<FormField
							control={form.control}
							name="host"
							render={({ field }) => (
								<FormItem>
									<FormLabel>Host</FormLabel>
									<FormControl>
										<Input placeholder="conan.example.com" autoComplete="off" {...field} />
									</FormControl>
									<FormMessage />
								</FormItem>
							)}
						/>
					</div>
					<FormField
						control={form.control}
						name="rcon_port"
						render={({ field }) => (
							<FormItem>
								<FormLabel>RCON port</FormLabel>
								<FormControl>
									<Input
										type="number"
										inputMode="numeric"
										min={1}
										max={65535}
										value={field.value}
										onChange={(e) => field.onChange(Number(e.target.value))}
										onBlur={field.onBlur}
										ref={field.ref}
										name={field.name}
									/>
								</FormControl>
								<FormMessage />
							</FormItem>
						)}
					/>
				</div>

				<FormField
					control={form.control}
					name="rcon_password"
					render={({ field }) => (
						<FormItem>
							<FormLabel>RCON password</FormLabel>
							<FormControl>
								<Input
									type="password"
									placeholder={server ? 'Re-enter password to save' : 'RCON password'}
									autoComplete="off"
									{...field}
								/>
							</FormControl>
							<FormDescription>Stored in your OS keychain — never written to disk.</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				<FormField
					control={form.control}
					name="admin_password"
					render={({ field }) => (
						<FormItem>
							<FormLabel>Admin password (optional)</FormLabel>
							<FormControl>
								<Input
									type="password"
									placeholder="In-game /MakeAdmin password"
									autoComplete="off"
									{...field}
								/>
							</FormControl>
							<FormDescription>
								Used by the "Promote to admin" prompt — leave blank if you don't use it.
							</FormDescription>
							<FormMessage />
						</FormItem>
					)}
				/>

				<div className="flex items-center justify-between gap-2 pt-2">
					<Button
						type="button"
						variant="outline"
						onClick={onTest}
						disabled={test.isPending}
						className={
							testStatus === 'ok'
								? 'border-emerald-500 text-emerald-600 dark:text-emerald-400'
								: testStatus === 'fail'
									? 'border-red-500 text-red-600 dark:text-red-400'
									: ''
						}
					>
						{test.isPending ? 'Testing…' : 'Test connection'}
					</Button>
					<div className="flex gap-2">
						<Button type="button" variant="ghost" onClick={onDone} disabled={save.isPending}>
							Cancel
						</Button>
						<Button type="submit" disabled={save.isPending}>
							{save.isPending ? 'Saving…' : server ? 'Save changes' : 'Add server'}
						</Button>
					</div>
				</div>
			</form>
		</Form>
	)
}
