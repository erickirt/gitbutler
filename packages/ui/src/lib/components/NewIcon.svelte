<script lang="ts" module>
	const modules = import.meta.glob<string>("../icons/*.svg", {
		query: "?raw",
		import: "default",
		eager: true,
	});

	const icons: Record<string, string> = Object.fromEntries(
		Object.entries(modules).map(([path, svg]) => [
			path.replace("../icons/", "").replace(".svg", ""),
			svg,
		]),
	);

	export type IconName = keyof typeof icons;
</script>

<script lang="ts">
	interface Props {
		name: IconName;
		size?: number;
		sizeUnit?: string;
		color?: string;
		class?: string;
	}

	const {
		name,
		size = 1,
		sizeUnit = "rem",
		color = "currentColor",
		class: className = "",
	}: Props = $props();

	const svg = $derived(icons[name]);
</script>

<span
	style="width: {size}{sizeUnit}; height: {size}{sizeUnit}; color: {color};"
	class="icon"
	class:className
>
	{@html svg}
</span>

<style>
	.icon {
		display: inline-flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
	}

	.icon :global(svg *) {
		vector-effect: non-scaling-stroke;
	}
</style>
