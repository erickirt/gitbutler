use anyhow::{bail, Context, Ok, Result};
use but_rebase::RebaseStep;
use gitbutler_command_context::CommandContext;
use gitbutler_commit::{commit_ext::CommitExt, commit_headers::HasCommitHeaders};
use gitbutler_oplog::{
    entry::{OperationKind, SnapshotDetails},
    OplogExt,
};
use gitbutler_oxidize::{GixRepositoryExt, ObjectIdExt, OidExt};
use gitbutler_project::access::WorktreeWritePermission;
use gitbutler_repo::{
    logging::{LogUntil, RepositoryExt},
    RepositoryExt as _,
};
use gitbutler_stack::StackId;
use gitbutler_workspace::branch_trees::{update_uncommited_changes, WorkspaceState};
use itertools::Itertools;

use crate::{
    reorder::{commits_order, reorder_stack},
    VirtualBranchesExt,
};

/// Squashes one or multiple commuits from a virtual branch into a destination commit
/// All of the commits involved have to be in the same stack
pub(crate) fn squash_commits(
    ctx: &CommandContext,
    stack_id: StackId,
    source_ids: Vec<git2::Oid>,
    desitnation_id: git2::Oid,
    perm: &mut WorktreeWritePermission,
) -> Result<git2::Oid> {
    // create a snapshot
    let snap = ctx.create_snapshot(SnapshotDetails::new(OperationKind::SquashCommit), perm)?;
    let result = do_squash_commits(ctx, stack_id, source_ids, desitnation_id, perm);
    // if result is error, restore from snapshot
    if result.is_err() {
        ctx.restore_snapshot(snap, perm)?;
    }
    result
}

fn do_squash_commits(
    ctx: &CommandContext,
    stack_id: StackId,
    mut source_ids: Vec<git2::Oid>,
    desitnation_id: git2::Oid,
    perm: &mut WorktreeWritePermission,
) -> Result<git2::Oid> {
    let old_workspace = WorkspaceState::create(ctx, perm.read_permission())?;
    let vb_state = ctx.project().virtual_branches();
    let stack = vb_state.get_stack_in_workspace(stack_id)?;
    let gix_repo = ctx.gix_repo()?;

    let default_target = vb_state.get_default_target()?;
    let merge_base = ctx
        .repo()
        .merge_base(stack.head_oid(&gix_repo)?.to_git2(), default_target.sha)?;

    // =========== Step 1: Reorder

    let order = commits_order(ctx, &stack)?;
    let mut updated_order = commits_order(ctx, &stack)?;
    // Remove source ids
    for branch in updated_order.series.iter_mut() {
        branch.commit_ids.retain(|id| !source_ids.contains(id));
    }
    // Put all source oids on top of (after) the destination oid
    for branch in updated_order.series.iter_mut() {
        if let Some(pos) = branch
            .commit_ids
            .iter()
            .position(|&id| id == desitnation_id)
        {
            branch.commit_ids.splice(pos..pos, source_ids.clone());
        }
    }
    let mapping = if order != updated_order {
        Some(reorder_stack(ctx, stack_id, updated_order, perm)?.commit_mapping)
    } else {
        None
    };

    // update source ids from the mapping if present
    if let Some(mapping) = mapping {
        for (_, old, new) in mapping.iter() {
            // if source_ids contains old, replace it with new
            if source_ids.contains(&old.to_git2()) {
                let index = source_ids
                    .iter()
                    .position(|id| id == &old.to_git2())
                    .unwrap();
                source_ids[index] = new.to_git2();
            }
        }
    };

    // =========== Step 2: Squash

    // stack was updated by reorder_stack, therefore it is reloaded
    let mut stack = vb_state.get_stack_in_workspace(stack_id)?;
    let branch_commit_oids = ctx.repo().l(
        stack.head_oid(&gix_repo)?.to_git2(),
        LogUntil::Commit(merge_base),
        false,
    )?;

    let branch_commits = branch_commit_oids
        .iter()
        .filter_map(|id| ctx.repo().find_commit(*id).ok())
        .collect_vec();

    // Find the new destination commit using the change id, error if not found
    let destination_change_id = ctx.repo().find_commit(desitnation_id)?.change_id();
    let destination_commit = branch_commits
        .iter()
        .find(|c| c.change_id() == destination_change_id)
        .context("Destination commit not found in the stack")?;

    // Find the new source commits using the change ids, error if not found
    let source_commits = source_ids
        .iter()
        .filter_map(|id| ctx.repo().find_commit(*id).ok())
        .map(|c| {
            branch_commits
                .iter()
                .find(|b| b.change_id() == c.change_id())
                .cloned()
                .context("Source commit not found in the stack")
        })
        .collect::<Result<Vec<_>>>()?;

    validate(
        ctx,
        &stack,
        &branch_commit_oids,
        &source_commits,
        destination_commit,
    )?;

    let final_tree = squash_tree(ctx, &source_commits, destination_commit)?;

    // Squash commit messages string separating with newlines
    let new_message = Some(destination_commit)
        .into_iter()
        .chain(source_commits.iter())
        .filter_map(|c| {
            let msg = c.message().unwrap_or_default();
            (!msg.trim().is_empty()).then_some(msg)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let parents: Vec<_> = destination_commit.parents().collect();

    // Create a new commit with the final tree
    let new_commit_oid = ctx
        .repo()
        .commit_with_signature(
            None,
            &destination_commit.author(),
            &destination_commit.author(),
            &new_message,
            &final_tree,
            &parents.iter().collect::<Vec<_>>(),
            destination_commit.gitbutler_headers(),
        )
        .context("Failed to create a squash commit")?;

    let mut steps: Vec<RebaseStep> = Vec::new();

    for head in stack.heads_by_commit(ctx.repo().find_commit(merge_base)?, &gix_repo) {
        steps.push(RebaseStep::Reference(but_core::Reference::Virtual(head)));
    }
    for oid in branch_commit_oids.iter().rev() {
        let commit = ctx.repo().find_commit(*oid)?;
        if source_ids.contains(oid) {
            // noop - skipping this
        } else if destination_commit.id() == *oid {
            steps.push(RebaseStep::Pick {
                commit_id: new_commit_oid.to_gix(),
                new_message: None,
            });
        } else {
            steps.push(RebaseStep::Pick {
                commit_id: oid.to_gix(),
                new_message: None,
            });
        }
        for head in stack.heads_by_commit(commit, &gix_repo) {
            steps.push(RebaseStep::Reference(but_core::Reference::Virtual(head)));
        }
    }

    let mut builder = but_rebase::Rebase::new(&gix_repo, merge_base.to_gix(), None)?;
    let builder = builder.steps(steps)?;
    builder.rebase_noops(false);
    let output = builder.rebase()?;

    let new_stack_head = output.top_commit.to_git2();

    stack.set_stack_head(&vb_state, &gix_repo, new_stack_head, None)?;

    let new_workspace = WorkspaceState::create(ctx, perm.read_permission())?;
    update_uncommited_changes(ctx, old_workspace, new_workspace, perm)?;
    crate::integration::update_workspace_commit(&vb_state, ctx)
        .context("failed to update gitbutler workspace")?;
    stack.set_heads_from_rebase_output(ctx, output.references)?;
    Ok(new_commit_oid)
}

fn validate(
    ctx: &CommandContext,
    stack: &gitbutler_stack::Stack,
    branch_commit_oids: &[git2::Oid],
    source_commits: &[git2::Commit<'_>],
    destination_commit: &git2::Commit<'_>,
) -> Result<()> {
    if source_commits
        .iter()
        .any(|s| s.id() == destination_commit.id())
    {
        bail!("cannot squash commit into itself")
    }

    for source_commit in source_commits {
        if !branch_commit_oids.contains(&source_commit.id()) {
            bail!("commit {} not in the stack", source_commit.id());
        }
    }

    if !branch_commit_oids.contains(&destination_commit.id()) {
        bail!("commit {} not in the stack", destination_commit.id());
    }

    for c in source_commits {
        if c.is_conflicted() {
            bail!("cannot squash conflicted source commit {}", c.id());
        }
    }

    if destination_commit.is_conflicted() {
        bail!("cannot squash into conflicted destination commit",);
    }

    let remote_commits = stack
        .branches()
        .iter()
        .flat_map(|b| b.commits(ctx, stack))
        .flat_map(|c| c.remote_commits)
        .map(|c| c.id())
        .collect_vec();

    if !stack.allow_rebasing {
        for source_commit in source_commits {
            if remote_commits.contains(&source_commit.id()) {
                bail!(
                    "Force push is now allowed. Source commits with id {} has already been pushed",
                    source_commit.id()
                );
            }
        }
        if remote_commits.contains(&destination_commit.id()) {
            bail!(
                "Force push is now allowed. Destination commit with id {} has already been pushed",
                destination_commit.id()
            );
        }
    }

    Ok(())
}

// Create a new tree that that has the source trees merged into the target tree
fn squash_tree<'a>(
    ctx: &'a CommandContext,
    source_commits: &[git2::Commit<'_>],
    destination_commit: &git2::Commit<'_>,
) -> Result<git2::Tree<'a>> {
    let mut final_tree_id = destination_commit.tree_id().to_gix();
    let gix_repo = ctx.gix_repo_for_merging()?;
    let (merge_options_fail_fast, conflict_kind) = gix_repo.merge_options_fail_fast()?;
    for source_commit in source_commits {
        let mut merge = gix_repo.merge_trees(
            source_commit.parent(0)?.tree_id().to_gix(),
            source_commit.tree_id().to_gix(),
            final_tree_id,
            gix_repo.default_merge_labels(),
            merge_options_fail_fast.clone(),
        )?;

        if merge.has_unresolved_conflicts(conflict_kind) {
            bail!("Merge failed with conflicts");
        }
        final_tree_id = merge.tree.write()?.detach();
    }
    let final_tree = ctx.repo().find_tree(final_tree_id.to_git2())?;
    Ok(final_tree)
}
