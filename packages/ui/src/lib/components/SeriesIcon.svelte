<script lang="ts">
	import NewIcon from "$components/NewIcon.svelte";
	import Tooltip from "$components/Tooltip.svelte";
	import { type NewIconName } from "$lib/icons/names";

	interface Props {
		single: boolean;
		origin?: boolean;
		size?: number;
	}

	const { single, origin, size }: Props = $props();

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
	<div class="stack-icon">
		<NewIcon name={getIcon()} {size} />
	</div>
</Tooltip>

<style>
	.stack-icon {
		display: flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		color: var(--clr-text-2);
	}
</style>
