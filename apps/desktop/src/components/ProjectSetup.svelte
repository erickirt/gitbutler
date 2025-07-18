<script lang="ts">
	import { goto } from '$app/navigation';
	import DecorativeSplitView from '$components/DecorativeSplitView.svelte';
	import KeysForm from '$components/KeysForm.svelte';
	import ProjectSetupTarget from '$components/ProjectSetupTarget.svelte';
	import ReduxResult from '$components/ReduxResult.svelte';
	import { PostHogWrapper } from '$lib/analytics/posthog';
	import newProjectSvg from '$lib/assets/illustrations/new-project.svg?raw';
	import BaseBranchService from '$lib/baseBranch/baseBranchService.svelte';
	import { platformName } from '$lib/platform/platform';
	import { ProjectsService } from '$lib/project/projectsService';
	import { inject } from '@gitbutler/shared/context';
	import Button from '@gitbutler/ui/Button.svelte';
	import type { RemoteBranchInfo } from '$lib/baseBranch/baseBranch';

	interface Props {
		projectId: string;
		remoteBranches: RemoteBranchInfo[];
	}

	const { projectId, remoteBranches }: Props = $props();

	const [projectsService, baseService, posthog] = inject(
		ProjectsService,
		BaseBranchService,
		PostHogWrapper
	);
	const projectResult = $derived(projectsService.getProject(projectId));
	const [setBaseBranchTarget] = baseService.setTarget;

	let selectedBranch = $state(['', '']);
	let loading = $state(false);

	async function setTarget() {
		if (!selectedBranch[0] || selectedBranch[0] === '') return;

		loading = true;
		try {
			await setBaseBranchTarget({
				projectId: projectId,
				branch: selectedBranch[0],
				pushRemote: selectedBranch[1]
			});
			goto(`/${projectId}/`, { invalidateAll: true });
		} finally {
			posthog.capture('Project Setup Complete');
			loading = false;
		}
	}
</script>

<DecorativeSplitView img={newProjectSvg}>
	{#if selectedBranch[0] && selectedBranch[0] !== '' && platformName !== 'windows'}
		{@const [remoteName, branchName] = selectedBranch[0].split(/\/(.*)/s)}
		<KeysForm {projectId} {remoteName} {branchName} disabled={loading} />
		<div class="actions">
			<Button kind="outline" disabled={loading} onclick={() => (selectedBranch[0] = '')}>
				Back
			</Button>
			<Button style="pop" {loading} onclick={setTarget} testId="accept-git-auth">Let's go!</Button>
		</div>
	{:else}
		<ReduxResult {projectId} result={projectResult.current}>
			{#snippet children(project)}
				<ProjectSetupTarget
					{projectId}
					projectName={project.title}
					{remoteBranches}
					onBranchSelected={async (branch) => {
						selectedBranch = branch;
					}}
				/>
			{/snippet}
		</ReduxResult>
	{/if}
</DecorativeSplitView>

<style lang="postcss">
	.actions {
		margin-top: 20px;
		text-align: right;
	}
</style>
