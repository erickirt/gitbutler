<script lang="ts">
	import Button from "$components/Button.svelte";
	import { type ModalType } from "$components/Modal.svelte";
	import NewIcon from "$components/NewIcon.svelte";
	import { type Snippet } from "svelte";

	interface Props {
		type?: ModalType;
		closeButton?: boolean;
		sticky?: boolean;
		closeButtonTestId?: string;
		children: Snippet;
		oncloseclick?: () => void;
	}

	const {
		type = "info",
		closeButton,
		sticky,
		closeButtonTestId,
		children,
		oncloseclick,
	}: Props = $props();
</script>

<div class="modal__header" class:sticky>
	{#if type === "warning"}
		<NewIcon name="warning" color="var(--clr-theme-warn-element)" />
	{/if}

	{#if type === "danger"}
		<NewIcon name="danger" color="var(--clr-theme-danger-element)" />
	{/if}

	{#if type === "success"}
		<NewIcon name="tick-circle" color="var(--clr-theme-safe-element)" />
	{/if}

	<h2 class="text-14 text-bold">
		{@render children()}
	</h2>

	{#if closeButton}
		<div class="close-btn">
			<Button
				type="button"
				testId={closeButtonTestId}
				kind="ghost"
				icon="cross"
				onclick={oncloseclick}
			></Button>
		</div>
	{/if}
</div>

<style lang="postcss">
	.modal__header {
		display: flex;
		position: relative;
		align-items: center;
		padding: 16px;
		gap: 8px;
		background-color: var(--clr-bg-1);

		&.sticky {
			border-bottom: 1px solid var(--clr-border-2);
		}
	}

	.close-btn {
		position: absolute;
		top: 10px;
		right: 10px;
	}
</style>
