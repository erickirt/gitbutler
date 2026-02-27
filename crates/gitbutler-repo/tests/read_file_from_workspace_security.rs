use std::{fs, path::Path};

use but_ctx::Context;
use but_settings::AppSettings;
use gitbutler_project as projects;
use gitbutler_repo::RepoCommands;
use gitbutler_testsupport::{commit_all, test_repository};

fn context_for_repo(repo: &git2::Repository) -> Context {
    let project = projects::Project::new_for_gitbutler_repo(
        repo.workdir().expect("workdir exists").to_path_buf(),
        projects::AuthKey::default(),
    );
    Context::new_from_legacy_project_and_settings(&project, AppSettings::default())
}

#[test]
fn allows_read_inside_worktree() {
    let (repo, _tmp) = test_repository();
    let workdir = repo.workdir().expect("workdir exists");
    fs::write(workdir.join("file.txt"), "hello from workspace").expect("write file");

    let ctx = context_for_repo(&repo);
    let info = ctx
        .read_file_from_workspace(Path::new("file.txt"))
        .expect("read file in workspace");

    assert_eq!(info.content, Some("hello from workspace".to_owned()));
}

#[test]
fn rejects_dotdot_traversal() {
    let (repo, _tmp) = test_repository();
    let workdir = repo.workdir().expect("workdir exists");
    let outside_path = workdir
        .parent()
        .expect("workdir has parent")
        .join("gitbutler-outside-secret.txt");
    fs::write(&outside_path, "outside").expect("write outside file");

    let traversal = format!(
        "../{}",
        outside_path
            .file_name()
            .expect("outside filename")
            .to_string_lossy()
    );

    let ctx = context_for_repo(&repo);
    let err = ctx
        .read_file_from_workspace(Path::new(&traversal))
        .expect_err("traversal must be rejected");

    assert!(
        err.to_string().contains("outside the worktree directory"),
        "{err:#}"
    );
}

#[cfg(unix)]
#[test]
fn rejects_symlink_escape() {
    let (repo, _tmp) = test_repository();
    let workdir = repo.workdir().expect("workdir exists");
    let outside_path = workdir
        .parent()
        .expect("workdir has parent")
        .join("gitbutler-symlink-target.txt");
    fs::write(&outside_path, "outside via symlink").expect("write outside file");
    std::os::unix::fs::symlink(&outside_path, workdir.join("link.txt")).expect("create symlink");

    let ctx = context_for_repo(&repo);
    let err = ctx
        .read_file_from_workspace(Path::new("link.txt"))
        .expect_err("symlink escape must be rejected");

    assert!(
        err.to_string().contains("outside the worktree directory"),
        "{err:#}"
    );
}

#[test]
fn keeps_deleted_file_fallback_behavior() {
    let (repo, _tmp) = test_repository();
    let workdir = repo.workdir().expect("workdir exists");
    fs::write(workdir.join("deleted.txt"), "tracked content").expect("write tracked file");
    commit_all(&repo);
    fs::remove_file(workdir.join("deleted.txt")).expect("delete file from workspace");

    let ctx = context_for_repo(&repo);
    let info = ctx
        .read_file_from_workspace(Path::new("deleted.txt"))
        .expect("deleted tracked file should still be readable from index/head fallback");

    assert_eq!(info.content, Some("tracked content".to_owned()));
}

#[test]
fn keeps_absolute_inside_worktree_behavior() {
    let (repo, _tmp) = test_repository();
    let workdir = repo.workdir().expect("workdir exists");
    let abs_path = workdir.join("absolute.txt");
    fs::write(&abs_path, "absolute read").expect("write file");

    let ctx = context_for_repo(&repo);
    let info = ctx
        .read_file_from_workspace(&abs_path)
        .expect("absolute in-worktree path should be readable");

    assert_eq!(info.content, Some("absolute read".to_owned()));
}
