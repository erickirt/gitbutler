//! These tests exercise the disconnect operation.
use std::collections::HashSet;

use anyhow::{Context, Result};
use but_graph::Graph;
use but_rebase::graph_rebase::{GraphExt, Step};
use but_testsupport::{git_status, visualize_commit_graph_all};
use gix::prelude::ObjectIdExt;

use crate::utils::{fixture_writable, standard_options};

#[test]
fn disconnect_and_remove_middle_commit_in_linear_history() -> Result<()> {
    let (repo, _tmpdir, meta) = fixture_writable("four-commits")?;

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
	* 120e3a9 (HEAD -> main) c
	* a96434e b
	* d591dfe a
	* 35b8235 base
	");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    let graph = Graph::from_head(&repo, &*meta, standard_options())?.validated()?;
    let mut editor = graph.to_editor(&repo)?;

    let b = repo.rev_parse_single("HEAD~")?.detach();
    let b_selector = editor
        .select_commit(b)
        .context("Failed to find commit b in editor graph")?;

    editor.disconnect(b_selector, b_selector)?;
    editor.replace(b_selector, Step::None)?;

    let outcome = editor.rebase()?;
    outcome.materialize()?;

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
	* 4de0144 (HEAD -> main) c
	* d591dfe a
	* 35b8235 base
	");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    Ok(())
}

#[test]
fn disconnect_and_remove_two_middle_commits_in_linear_history() -> Result<()> {
    let (repo, _tmpdir, meta) = fixture_writable("four-commits")?;

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
	* 120e3a9 (HEAD -> main) c
	* a96434e b
	* d591dfe a
	* 35b8235 base
	");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    let graph = Graph::from_head(&repo, &*meta, standard_options())?.validated()?;
    let mut editor = graph.to_editor(&repo)?;

    let b = repo.rev_parse_single("HEAD~")?.detach();
    let b_selector = editor
        .select_commit(b)
        .context("Failed to find commit b in editor graph")?;
    let a = repo.rev_parse_single("HEAD~2")?.detach();
    let a_selector = editor
        .select_commit(a)
        .context("Failed to find commit a in editor graph")?;

    editor.disconnect(b_selector, a_selector)?;
    editor.replace(b_selector, Step::None)?;
    editor.replace(a_selector, Step::None)?;

    let outcome = editor.rebase()?;
    outcome.materialize()?;

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @"
    * f55e07c (HEAD -> main) c
    * 35b8235 base
    ");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    Ok(())
}

#[test]
fn disconnect_and_remove_commit_in_merge_history_rewires_children() -> Result<()> {
    let (repo, _tmpdir, meta) = fixture_writable("merge-in-the-middle")?;

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
    * e8ee978 (HEAD -> with-inner-merge) on top of inner merge
    *   2fc288c Merge branch 'B' into with-inner-merge
    |\  
    | * 984fd1c (B) C: new file with 10 lines
    * | add59d2 (A) A: 10 lines on top
    |/  
    * 8f0d338 (tag: base, main) base
    ");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    let graph = Graph::from_head(&repo, &*meta, standard_options())?.validated()?;
    let mut editor = graph.to_editor(&repo)?;

    let a = repo.rev_parse_single("A")?.detach();
    let a_selector = editor
        .select_commit(a)
        .context("Failed to find commit a in editor graph")?;

    editor.disconnect(a_selector, a_selector)?;
    editor.replace(a_selector, Step::None)?;

    let outcome = editor.rebase()?;
    outcome.materialize()?;

    let a_now = repo.rev_parse_single("A")?.detach();
    let base = repo.rev_parse_single("base")?.detach();
    assert_eq!(a_now, base, "A should now point to base after disconnect");

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
    * dde6cc8 (HEAD -> with-inner-merge) on top of inner merge
    *   5f962e2 Merge branch 'B' into with-inner-merge
    |\  
    | * 984fd1c (B) C: new file with 10 lines
    |/  
    * 8f0d338 (tag: base, main, A) base
    ");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    Ok(())
}

#[test]
fn disconnect_and_remove_merge_with_two_parents_and_two_children() -> Result<()> {
    let (repo, _tmpdir, meta) = fixture_writable("merge-with-two-children")?;

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
    *   d1cc4c7 (HEAD -> with-two-children) tip
    |\  
    | * ce6aca9 (C2) C2: second child
    * | f94f259 (C1) C1: first child
    |/  
    *   c5d1178 (M) M: merge two parents
    |\  
    | * 392a8f8 (P2) P2: second merge parent
    * | bc0e772 (P1) P1: first merge parent
    |/  
    * 7674a5e (tag: base, main) base
    ");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    let graph = Graph::from_head(&repo, &*meta, standard_options())?.validated()?;
    let mut editor = graph.to_editor(&repo)?;

    let merge = repo.rev_parse_single("M")?.detach();
    let merge_selector = editor
        .select_commit(merge)
        .context("Failed to find merge commit M in editor graph")?;

    editor.disconnect(merge_selector, merge_selector)?;
    editor.replace(merge_selector, Step::None)?;

    let outcome = editor.rebase()?;
    outcome.materialize()?;

    let p1 = repo.rev_parse_single("P1")?.detach();
    let p2 = repo.rev_parse_single("P2")?.detach();
    let expected_parents = HashSet::from([p1, p2]);

    let c1 = repo.rev_parse_single("C1")?.detach();
    let c1_commit = but_core::Commit::from_id(c1.attach(&repo))?;
    let c1_parents = c1_commit
        .inner
        .parents
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(
        c1_parents, expected_parents,
        "C1 should have both merge parents after removing M"
    );

    let c2 = repo.rev_parse_single("C2")?.detach();
    let c2_commit = but_core::Commit::from_id(c2.attach(&repo))?;
    let c2_parents = c2_commit
        .inner
        .parents
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(
        c2_parents, expected_parents,
        "C2 should have both merge parents after removing M"
    );

    insta::assert_snapshot!(visualize_commit_graph_all(&repo)?, @r"
    *   f914957 (HEAD -> with-two-children) tip
    |\  
    | *   72b8072 (C2) C2: second child
    | |\  
    * | \   d8cc9ec (C1) C1: first child
    |\ \ \  
    | |/ /  
    |/| /   
    | |/    
    | * 392a8f8 (P2) P2: second merge parent
    * | bc0e772 (P1, M) P1: first merge parent
    |/  
    * 7674a5e (tag: base, main) base
    ");
    insta::assert_snapshot!(git_status(&repo)?, @"");

    Ok(())
}
