use git_ghosts::detect_ghost_files;

#[test]
fn test_detect_ghost_files_basic() {
    let dir = std::env::temp_dir().join(format!(
        "gg-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let run = |args: &[&str]| {
        std::process::Command::new(args[0])
            .args(&args[1..])
            .current_dir(&dir)
            .output()
            .expect("command failed")
    };

    run(&["git", "init"]);
    run(&["git", "config", "user.email", "test@example.com"]);
    run(&["git", "config", "user.name", "TestUser"]);

    std::fs::write(dir.join("ghost.txt"), "Hello ghost").unwrap();
    std::fs::write(dir.join("keep.txt"), "Hello keep").unwrap();

    run(&["git", "add", "."]);
    run(&["git", "commit", "-m", "add files"]);

    std::fs::remove_file(dir.join("ghost.txt")).unwrap();

    run(&["git", "add", "-A"]);
    run(&["git", "commit", "-m", "delete ghost.txt"]);

    let ghosts = detect_ghost_files(&dir).unwrap();

    assert_eq!(ghosts.len(), 1);
    assert_eq!(ghosts[0].file_path, "ghost.txt");
    assert!(!ghosts.iter().any(|g| g.file_path == "keep.txt"));
    assert_eq!(ghosts[0].original_file_size_bytes, 11);
    assert!(!ghosts[0].deletion_commit_hash.is_empty());
    assert_eq!(ghosts[0].author, "TestUser");
}

#[test]
fn test_detect_ghost_files_non_git_repo() {
    let dir = std::env::temp_dir().join(format!(
        "gg-test-nongit-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();

    let result = detect_ghost_files(&dir);
    assert!(result.is_err());
}
