<script lang="ts">
	import { StackService } from '$lib/stacks/stackService.svelte';
	import { getContext } from '@gitbutler/shared/context';
	import ContextMenu from '@gitbutler/ui/ContextMenu.svelte';
	import ContextMenuItem from '@gitbutler/ui/ContextMenuItem.svelte';
	import ContextMenuSection from '@gitbutler/ui/ContextMenuSection.svelte';
	import Icon from '@gitbutler/ui/Icon.svelte';

	type Props = {
		projectId: string;
		stackId: string;
	};

	const { projectId, stackId }: Props = $props();

	const stackService = getContext(StackService);

	let trigger = $state<HTMLElement>();
	let contextMenu = $state<ContextMenu>();
	let isOpen = $state(false);
</script>

<button
	class="menu-button"
	class:menu-open={isOpen}
	onclick={(e) => {
		e.preventDefault();
		e.stopPropagation();
		contextMenu?.toggle();
	}}
	bind:this={trigger}
	type="button"
>
	<Icon name="kebab" />
</button>
<ContextMenu
	bind:this={contextMenu}
	leftClickTrigger={trigger}
	ontoggle={(isOpen) => (isOpen = isOpen)}
	side="bottom"
>
	<ContextMenuSection>
		<ContextMenuItem
			label="Unapply Stack"
			keyboardShortcut="$mod+X"
			onclick={async () => {
				await stackService.unapply(projectId, stackId);
				contextMenu?.close();
			}}
		/>
		<ContextMenuItem
			label="Rename"
			keyboardShortcut="$mod+R"
			onclick={() => {
				contextMenu?.close();
			}}
		/>
	</ContextMenuSection>
</ContextMenu>

<style lang="postcss">
	.menu-button {
		display: flex;
		color: var(--clr-text-2);
		padding: 4px 2px;
		margin-left: -2px;

		&.menu-open,
		&:hover {
			color: var(--clr-text-1);
		}
	}
</style>
