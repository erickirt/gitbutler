<script module lang="ts">
	import NewIcon from "$components/NewIcon.svelte";
	import { defineMeta } from "@storybook/addon-svelte-csf";
	import type { IconName } from "$components/NewIcon.svelte";

	const iconModules = import.meta.glob<string>("../../lib/icons/*.svg", {
		query: "?raw",
		import: "default",
		eager: true,
	});
	let allIconNames = Object.keys(iconModules).map((p) =>
		p.replace("../../lib/icons/", "").replace(".svg", ""),
	) as IconName[];

	const { Story } = defineMeta({
		title: "Basic / NewIcon",
		component: NewIcon,
		args: {
			name: "pr" as IconName,
			size: 1,
			sizeUnit: "rem",
			color: "currentColor",
		},
		argTypes: {
			name: {
				options: allIconNames,
				control: { type: "select" },
			},
			size: {
				control: { type: "number" },
			},
			sizeUnit: {
				options: ["rem", "px", "%", "em"],
				control: { type: "select" },
			},
			color: {
				control: { type: "color" },
			},
		},
	});
</script>

<Story name="Playground">
	{#snippet template(args)}
		<NewIcon name={args.name} size={args.size} color={args.color} />
	{/snippet}
</Story>

<Story name="All Icons">
	{#snippet template(args)}
		<div class="icons">
			{#each allIconNames as name}
				<div class="icon-item">
					<NewIcon {name} size={args.size} color={args.color} />
					<span class="text-11 icon-label">{name}</span>
				</div>
			{/each}
		</div>
	{/snippet}
</Story>

<style>
	.icons {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
		padding: 32px;
		gap: 16px;
	}

	.icon-item {
		display: flex;
		flex-direction: column;
		gap: 12px;
		color: var(--clr-text-1);
	}

	.icon-label {
		color: var(--clr-text-2);
	}
</style>
