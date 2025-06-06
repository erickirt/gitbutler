use super::*;

#[test]
fn twice() {
    let data_dir = paths::data_dir();
    let projects = projects::Controller::from_path(data_dir.path());

    let test_project = TestProject::default();

    {
        let project = projects
            .add(test_project.path(), None, None)
            .expect("failed to add project");
        let ctx = CommandContext::open(&project, AppSettings::default()).unwrap();

        gitbutler_branch_actions::set_base_branch(
            &ctx,
            &"refs/remotes/origin/master".parse().unwrap(),
            false,
            ctx.project().exclusive_worktree_access().write_permission(),
        )
        .unwrap();
        assert!(gitbutler_branch_actions::list_virtual_branches(&ctx)
            .unwrap()
            .branches
            .is_empty());
        projects.delete(project.id).unwrap();
        gitbutler_branch_actions::list_virtual_branches(&ctx).unwrap_err();
    }

    {
        let project = projects.add(test_project.path(), None, None).unwrap();
        let ctx = CommandContext::open(&project, AppSettings::default()).unwrap();
        gitbutler_branch_actions::set_base_branch(
            &ctx,
            &"refs/remotes/origin/master".parse().unwrap(),
            false,
            ctx.project().exclusive_worktree_access().write_permission(),
        )
        .unwrap();

        // even though project is on gitbutler/workspace, we should not import it
        assert!(gitbutler_branch_actions::list_virtual_branches(&ctx)
            .unwrap()
            .branches
            .is_empty());
    }
}

#[test]
fn dirty_non_target() {
    // a situation when you initialize project while being on the local verison of the master
    // that has uncommited changes.
    let Test { repo, ctx, .. } = &Test::default();

    repo.checkout(&"refs/heads/some-feature".parse().unwrap());

    fs::write(repo.path().join("file.txt"), "content").unwrap();

    gitbutler_branch_actions::set_base_branch(
        ctx,
        &"refs/remotes/origin/master".parse().unwrap(),
        false,
        ctx.project().exclusive_worktree_access().write_permission(),
    )
    .unwrap();

    let list_result = gitbutler_branch_actions::list_virtual_branches(ctx).unwrap();
    let branches = list_result.branches;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].files.len(), 1);
    assert_eq!(branches[0].files[0].hunks.len(), 1);
    assert!(branches[0].upstream.is_none());
    assert_eq!(branches[0].name, "some-feature");
}

#[test]
fn dirty_target() {
    // a situation when you initialize project while being on the local verison of the master
    // that has uncommited changes.
    let Test { repo, ctx, .. } = &Test::default();

    fs::write(repo.path().join("file.txt"), "content").unwrap();

    gitbutler_branch_actions::set_base_branch(
        ctx,
        &"refs/remotes/origin/master".parse().unwrap(),
        false,
        ctx.project().exclusive_worktree_access().write_permission(),
    )
    .unwrap();

    let list_result = gitbutler_branch_actions::list_virtual_branches(ctx).unwrap();
    let branches = list_result.branches;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].files.len(), 1);
    assert_eq!(branches[0].files[0].hunks.len(), 1);
    assert!(branches[0].upstream.is_none());
    assert_eq!(branches[0].name, "master");
}

#[test]
fn commit_on_non_target_local() {
    let Test { repo, ctx, .. } = &Test::default();

    repo.checkout(&"refs/heads/some-feature".parse().unwrap());
    fs::write(repo.path().join("file.txt"), "content").unwrap();
    repo.commit_all("commit on target");

    gitbutler_branch_actions::set_base_branch(
        ctx,
        &"refs/remotes/origin/master".parse().unwrap(),
        false,
        ctx.project().exclusive_worktree_access().write_permission(),
    )
    .unwrap();

    let list_result = gitbutler_branch_actions::list_virtual_branches(ctx).unwrap();
    let branches = list_result.branches;
    assert_eq!(branches.len(), 1);
    assert!(branches[0].files.is_empty());
    assert_eq!(branches[0].series[0].clone().unwrap().patches.len(), 1);
    assert!(branches[0].upstream.is_none());
    assert_eq!(branches[0].name, "some-feature");
}

#[test]
fn commit_on_non_target_remote() {
    let Test { repo, ctx, .. } = &Test::default();

    repo.checkout(&"refs/heads/some-feature".parse().unwrap());
    fs::write(repo.path().join("file.txt"), "content").unwrap();
    repo.commit_all("commit on target");
    repo.push_branch(&"refs/heads/some-feature".parse().unwrap());

    gitbutler_branch_actions::set_base_branch(
        ctx,
        &"refs/remotes/origin/master".parse().unwrap(),
        false,
        ctx.project().exclusive_worktree_access().write_permission(),
    )
    .unwrap();

    let list_result = gitbutler_branch_actions::list_virtual_branches(ctx).unwrap();
    let branches = list_result.branches;
    assert_eq!(branches.len(), 1);
    assert!(branches[0].files.is_empty());
    assert_eq!(branches[0].series[0].clone().unwrap().patches.len(), 1);
    assert!(branches[0].upstream.is_some());
    assert_eq!(branches[0].name, "some-feature");
}

#[test]
fn commit_on_target() {
    let Test { repo, ctx, .. } = &Test::default();

    fs::write(repo.path().join("file.txt"), "content").unwrap();
    repo.commit_all("commit on target");

    gitbutler_branch_actions::set_base_branch(
        ctx,
        &"refs/remotes/origin/master".parse().unwrap(),
        false,
        ctx.project().exclusive_worktree_access().write_permission(),
    )
    .unwrap();

    let list_result = gitbutler_branch_actions::list_virtual_branches(ctx).unwrap();
    let branches = list_result.branches;
    assert_eq!(branches.len(), 1);
    assert!(branches[0].files.is_empty());
    assert_eq!(branches[0].series[0].clone().unwrap().patches.len(), 1);
    assert!(branches[0].upstream.is_none());
    assert_eq!(branches[0].name, "master");
}

#[test]
fn submodule() {
    let Test { repo, ctx, .. } = &Test::default();

    let test_project = TestProject::default();
    let submodule_url: gitbutler_url::Url =
        test_project.path().display().to_string().parse().unwrap();
    repo.add_submodule(&submodule_url, path::Path::new("submodule"));

    gitbutler_branch_actions::set_base_branch(
        ctx,
        &"refs/remotes/origin/master".parse().unwrap(),
        false,
        ctx.project().exclusive_worktree_access().write_permission(),
    )
    .unwrap();

    let list_result = gitbutler_branch_actions::list_virtual_branches(ctx).unwrap();
    let branches = list_result.branches;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].files.len(), 1);
    assert_eq!(branches[0].files[0].hunks.len(), 1);
}
