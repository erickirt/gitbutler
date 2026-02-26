<script lang="ts">
	import NewIcon from "$components/NewIcon.svelte";
	import Tooltip from "$components/Tooltip.svelte";
	import { type NewIconName } from "$lib/data/newIconNames";

	interface Props {
		single: boolean;
		origin?: boolean;
		outlined?: boolean;
	}

	const { single, origin, outlined }: Props = $props();

	function getIcon(): NewIconName {
		if (origin) {
			return "home";
		}

		if (single) {
			return "branch";
		}

		return "link";
	}
</script>

<Tooltip text={single ? "Single branch" : "Multiple branches"}>
	<div class="stack-icon" class:outlined>
		<NewIcon name={getIcon()} />
	</div>
</Tooltip>

<style>
	.stack-icon {
		display: flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-s);
		box-shadow: inset 0 0 0 1px var(--clr-border-2);
		color: var(--clr-text-2);
	}
</style>
