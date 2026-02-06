use airdb_lib::engine::github::{GitSync, GitHubClient};
use std::path::Path;
use std::fs;

#[test]
fn test_sync_conflict_flow() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup directories
    let root = tempfile::tempdir()?;
    let origin_path = root.path().join("origin.git");
    let alice_path = root.path().join("alice");
    let bob_path = root.path().join("bob");

    // 2. Create bare origin repo
    let _origin = git2::Repository::init_bare(&origin_path)?;
    let origin_url = format!("file://{}", origin_path.display());

    // 3. Alice inits and pushes first commit
    fs::create_dir(&alice_path)?;
    let alice_sync = GitSync::new(&alice_path, "dummy_token"); // Token ignored for file://
    
    // Init local repo
    let repo = GitHubClient::init_repo(&alice_path)?;
    GitHubClient::add_remote(&repo, "origin", &origin_url)?;
    
    // Create initial file
    fs::write(alice_path.join("README.md"), "# AirDB Project")?;
    
    // Commit and Push
    println!("Alice committing initial...");
    alice_sync.commit("Initial commit", "Alice", "alice@example.com")?;
    println!("Alice pushing initial...");
    alice_sync.push("main")?;

    // 4. Bob clones
    println!("Bob cloning...");
    let _bob_repo = GitHubClient::clone_repo(&origin_url, &bob_path, "dummy_token")?;
    let bob_sync = GitSync::new(&bob_path, "dummy_token");

    // 5. Alice makes a change
    fs::write(alice_path.join("data.txt"), "Alice's data")?;
    println!("Alice committing update...");
    alice_sync.commit("Alice update", "Alice", "alice@example.com")?;
    println!("Alice pushing update...");
    alice_sync.push("main")?;

    // 6. Bob makes a conflicting change
    fs::write(bob_path.join("data.txt"), "Bob's data")?;
    println!("Bob committing update...");
    bob_sync.commit("Bob update", "Bob", "bob@example.com")?;

    // 7. Bob pulls - Should fail with Conflict
    println!("Bob pulling (expecting conflict)...");
    let result = bob_sync.pull("main");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Merge conflicts detected"));
    assert!(err.to_string().contains("data.txt"));

    // 8. Bob lists conflicts
    let conflicts = bob_sync.list_conflicts()?;
    assert_eq!(conflicts.len(), 1);
    assert!(conflicts[0].contains("data.txt"));

    // 9. Bob resolves conflict (Choosing Theirs/Alice's)
    bob_sync.resolve_conflict("data.txt", "theirs")?;
    
    // Verify content is Alice's
    let content = fs::read_to_string(bob_path.join("data.txt"))?;
    assert_eq!(content, "Alice's data");

    // 10. Bob commits merge and pushes
    // Note: resolve_conflict adds to index, but we need to commit the merge
    // In our CLI flow, the user runs 'sync resolved' or similar, but GitSync::pull 
    // puts us in a merging state. We just need to commit.
    bob_sync.commit("Merge conflict resolved", "Bob", "bob@example.com")?;
    bob_sync.push("main")?;

    // 11. Alice pulls - Should be fast-forward
    alice_sync.pull("main")?;
    let content = fs::read_to_string(alice_path.join("data.txt"))?;
    assert_eq!(content, "Alice's data");

    Ok(())
}
