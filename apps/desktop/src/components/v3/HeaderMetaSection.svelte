<script lang="ts">
	import BranchLaneContextMenu from '$components/BranchLaneContextMenu.svelte';
	import { PatchSeries } from '$lib/branches/branch';
	import Button from '@gitbutler/ui/Button.svelte';
	import ContextMenu from '@gitbutler/ui/ContextMenu.svelte';
	import SeriesLabelsRow from '@gitbutler/ui/SeriesLabelsRow.svelte';

	interface Props {
		projectId: string;
		series: (PatchSeries | Error)[];
		onCollapseButtonClick: () => void;
		stackId?: string;
	}

	const { projectId, series, onCollapseButtonClick }: Props = $props();

	let contextMenu = $state<ReturnType<typeof ContextMenu>>();
	let kebabButtonEl: HTMLButtonElement | undefined = $state();
	let isContextMenuOpen = $state(false);
</script>

<div class="stack-meta">
	<div class="stack-meta-top">
		<SeriesLabelsRow series={series.map((s) => s.name)} />

		<Button
			bind:el={kebabButtonEl}
			activated={isContextMenuOpen}
			kind="ghost"
			icon="kebab"
			size="tag"
			onclick={() => {
				contextMenu?.toggle();
			}}
		/>
		<BranchLaneContextMenu
			{projectId}
			bind:contextMenuEl={contextMenu}
			trigger={kebabButtonEl}
			onCollapse={onCollapseButtonClick}
			ontoggle={(isOpen) => (isContextMenuOpen = isOpen)}
		/>
	</div>
</div>

<style lang="postcss">
	.stack-meta {
		display: flex;
		flex-direction: column;
		align-items: start;
		align-items: center;
		width: 100%;
		gap: 4px;
		border: 1px solid var(--clr-border-2);
		border-top: none;
		background-color: var(--clr-bg-1);
	}

	.stack-meta-top {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 12px;
		gap: 8px;
	}
</style>
