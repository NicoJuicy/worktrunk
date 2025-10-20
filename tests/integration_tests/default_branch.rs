use crate::common::TestRepo;
use worktrunk::git::Repository;

#[test]
fn test_get_default_branch_with_origin_head() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");
    repo.setup_remote("main");

    // origin/HEAD should be set automatically by setup_remote
    assert!(repo.has_origin_head());

    // Test that we can get the default branch
    let branch = Repository::at(repo.root_path())
        .default_branch()
        .expect("Failed to get default branch");
    assert_eq!(branch, "main");
}

#[test]
fn test_get_default_branch_without_origin_head() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");
    repo.setup_remote("main");

    // Clear origin/HEAD to force remote query
    repo.clear_origin_head();
    assert!(!repo.has_origin_head());

    // Should still work by querying remote
    let branch = Repository::at(repo.root_path())
        .default_branch()
        .expect("Failed to get default branch");
    assert_eq!(branch, "main");

    // Verify that origin/HEAD is now cached
    assert!(repo.has_origin_head());
}

#[test]
fn test_get_default_branch_caches_result() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");
    repo.setup_remote("main");

    // Clear origin/HEAD
    repo.clear_origin_head();
    assert!(!repo.has_origin_head());

    // First call queries remote and caches
    Repository::at(repo.root_path())
        .default_branch()
        .expect("Failed to get default branch");
    assert!(repo.has_origin_head());

    // Second call uses cache (fast path)
    let branch = Repository::at(repo.root_path())
        .default_branch()
        .expect("Failed to get default branch on second call");
    assert_eq!(branch, "main");
}

#[test]
fn test_get_default_branch_no_remote() {
    let repo = TestRepo::new();
    repo.commit("Initial commit");

    // No remote configured, should fail
    let result = Repository::at(repo.root_path()).default_branch();
    assert!(result.is_err());
}

#[test]
fn test_get_default_branch_with_custom_remote() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");
    repo.setup_custom_remote("upstream", "main");

    // Test that we can get the default branch from a custom remote
    let branch = Repository::at(repo.root_path())
        .default_branch()
        .expect("Failed to get default branch");
    assert_eq!(branch, "main");
}

#[test]
fn test_primary_remote_detects_custom_remote() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");
    repo.setup_custom_remote("upstream", "develop");

    // Test that primary_remote detects the custom remote name
    let remote = Repository::at(repo.root_path())
        .primary_remote()
        .expect("Failed to get primary remote");
    assert_eq!(remote, "upstream");
}

#[test]
fn test_branch_exists_with_custom_remote() {
    let mut repo = TestRepo::new();
    repo.commit("Initial commit");
    repo.setup_custom_remote("upstream", "main");

    let git_repo = Repository::at(repo.root_path());

    // Should find the branch on the custom remote
    assert!(
        git_repo
            .branch_exists("main")
            .expect("Failed to check branch")
    );

    // Should not find non-existent branch
    assert!(
        !git_repo
            .branch_exists("nonexistent")
            .expect("Failed to check branch")
    );
}
