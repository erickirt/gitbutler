<script lang="ts">
	import IssueUpdate from '$lib/components/chat/IssueUpdate.svelte';
	import Message from '$lib/components/chat/Message.svelte';
	import PatchStatus from '$lib/components/chat/PatchStatus.svelte';
	import PatchVersion from '$lib/components/chat/PatchVersion.svelte';
	import type { ChatEvent, PatchEvent } from '@gitbutler/shared/patchEvents/types';

	interface Props {
		highlightedMessageUuid: string | undefined;
		projectId: string;
		projectSlug: string;
		changeId: string;
		event: PatchEvent;
		replyTo: (chatEvent: ChatEvent) => void;
		scrollToMessage: (uuid: string) => void;
	}

	const {
		event,
		projectId,
		projectSlug,
		changeId,
		highlightedMessageUuid,
		replyTo,
		scrollToMessage
	}: Props = $props();
</script>

{#if event.eventType === 'chat'}
	<Message
		{projectSlug}
		{projectId}
		{changeId}
		{event}
		highlight={highlightedMessageUuid === event.object.uuid}
		onReply={() => replyTo(event)}
		{scrollToMessage}
	/>
{:else if event.eventType === 'issue_status'}
	<IssueUpdate {event} />
{:else if event.eventType === 'patch_version'}
	<PatchVersion {event} />
{:else if event.eventType === 'patch_status'}
	<PatchStatus {event} />
{/if}
