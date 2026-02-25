<script lang="ts">
	import NewIcon from "$components/NewIcon.svelte";
	import SkeletonBone from "$components/SkeletonBone.svelte";
	import Tooltip from "$components/Tooltip.svelte";
	import { type NewIconName } from "$lib/data/newIconNames";
	import type { ComponentColorType } from "$lib/utils/colorTypes";
	import type { Snippet } from "svelte";

	interface Props {
		testId?: string;
		style?: ComponentColorType;
		kind?: "solid" | "soft";
		size?: "icon" | "tag";
		class?: string;
		icon?: NewIconName;
		tooltip?: string;
		skeleton?: boolean;
		skeletonWidth?: string;
		children?: Snippet;
		onclick?: (e: MouseEvent) => void;
		reversedDirection?: boolean;
	}

	const {
		testId,
		style = "gray",
		kind = "solid",
		size = "icon",
		class: className = "",
		icon,
		tooltip,
		skeleton,
		skeletonWidth,
		children,
		onclick,
		reversedDirection,
	}: Props = $props();
</script>

{#if skeleton}
	<SkeletonBone
		radius="3rem"
		width={skeletonWidth ?? (size === "icon" ? "var(--size-icon)" : "var(--size-tag)")}
		height={size === "icon" ? "var(--size-icon)" : "var(--size-tag)"}
	/>
{:else}
	<Tooltip text={tooltip}>
		<div
			role="presentation"
			data-testid={testId}
			class="badge {style} {kind} {size}-size {className}"
			class:reversed={reversedDirection}
			{onclick}
		>
			{#if children}
				<span class="badge__label text-11 text-bold">{@render children()}</span>
			{/if}
			{#if icon}
				<i class="badge__icon">
					<NewIcon name={icon} />
				</i>
			{/if}
		</div>
	</Tooltip>
{/if}

<style lang="postcss">
	.badge {
		display: inline-flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		border-radius: 30px;
		text-align: center;

		/* SOLID */
		&.gray.solid {
			background-color: var(--clr-theme-gray-element);
			color: var(--clr-theme-gray-on-element);
		}

		&.pop.solid {
			background-color: var(--clr-theme-pop-element);
			color: var(--clr-theme-pop-on-element);
		}

		&.safe.solid {
			background-color: var(--clr-theme-safe-element);
			color: var(--clr-theme-safe-on-element);
		}

		&.warning.solid {
			background-color: var(--clr-theme-warn-element);
			color: var(--clr-theme-warn-on-element);
		}

		&.danger.solid {
			background-color: var(--clr-theme-danger-element);
			color: var(--clr-theme-danger-on-element);
		}

		&.purple.solid {
			background-color: var(--clr-theme-purple-element);
			color: var(--clr-theme-purple-on-element);
		}

		/* SOFT */
		&.gray.soft {
			background-color: var(--clr-theme-gray-soft);
			color: var(--clr-theme-gray-on-soft);
		}

		&.pop.soft {
			background-color: var(--clr-theme-pop-soft);
			color: var(--clr-theme-pop-on-soft);
		}

		&.safe.soft {
			background-color: var(--clr-theme-safe-soft);
			color: var(--clr-theme-safe-on-soft);
		}

		&.warning.soft {
			background-color: var(--clr-theme-warn-soft);
			color: var(--clr-theme-warn-on-soft);
		}

		&.danger.soft {
			background-color: var(--clr-theme-danger-soft);
			color: var(--clr-theme-danger-on-soft);
		}

		&.purple.soft {
			background-color: var(--clr-theme-purple-soft);
			color: var(--clr-theme-purple-on-soft);
		}

		/* SIZE */
		&.icon-size {
			padding: 3px 6px;
			gap: 2px;
		}

		&.tag-size {
			height: var(--size-tag);
			padding: 0 8px;
			gap: 3px;
		}

		&.reversed {
			flex-direction: row-reverse;
		}
	}

	.badge__label {
		display: flex;
		line-height: var(--size-icon);
		line-height: 1;
		white-space: nowrap;
	}

	.badge__icon {
		display: flex;
		opacity: 0.7;
	}

	@supports (text-box: trim-both ex alphabetic) {
		.badge__label {
			text-box: trim-both ex alphabetic;
		}
	}

	@support not (text-box: trim-both ex alphabetic) {
		.badge__label {
			padding-top: 1px;
		}
	}
</style>
