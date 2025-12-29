use mug::ui::{UnicodeFormatter, CommitInfo, DiffHunk, DiffLine};

fn main() {
    println!("\n=== MUG Beautiful Output Examples ===\n");

    let formatter = UnicodeFormatter::new(true, true);

    // Demo 1: Commit Log
    println!("--- 1. COMMIT LOG OUTPUT ---");
    let commits = vec![
        CommitInfo {
            hash: "a1b2c3d4e5f6g7h8".to_string(),
            author: "Alice <alice@example.com>".to_string(),
            date: "2025-12-29 14:32:15".to_string(),
            message: "Add beautiful output formatting".to_string(),
            is_head: true,
            branch: Some("main".to_string()),
        },
        CommitInfo {
            hash: "f7h8i9j0k1l2m3n4".to_string(),
            author: "Bob <bob@example.com>".to_string(),
            date: "2025-12-28 10:15:42".to_string(),
            message: "Implement Unicode formatter".to_string(),
            is_head: false,
            branch: None,
        },
        CommitInfo {
            hash: "m3n4o5p6q7r8s9t0".to_string(),
            author: "Charlie <charlie@example.com>".to_string(),
            date: "2025-12-27 09:05:20".to_string(),
            message: "Initial project setup".to_string(),
            is_head: false,
            branch: None,
        },
    ];

    println!("{}\n", formatter.format_log(&commits));

    // Demo 2: Status
    println!("--- 2. REPOSITORY STATUS ---");
    let changes = vec![
        ("src/ui/formatter.rs".to_string(), 'M'),
        ("examples/formatter_demo.rs".to_string(), 'A'),
        ("old_code.rs".to_string(), 'D'),
        ("README.md".to_string(), 'M'),
    ];
    println!("{}\n", formatter.format_status("feature/beautiful-output", &changes));

    // Demo 3: Branch List
    println!("--- 3. BRANCH LISTING ---");
    let branches = vec![
        "main".to_string(),
        "develop".to_string(),
        "feature/beautiful-output".to_string(),
        "hotfix/security-patch".to_string(),
        "release/1.0.0".to_string(),
    ];
    println!("{}\n", formatter.format_branch_list("feature/beautiful-output", &branches));

    // Demo 4: Progress Bar
    println!("--- 4. PROGRESS BARS ---");
    for percent in [0, 25, 50, 75, 100] {
        println!("  {}", formatter.format_progress_bar(percent, 100));
    }
    println!();

    // Demo 5: Messages
    println!("--- 5. COLORED MESSAGES ---");
    println!("  {}", formatter.format_success("Changes committed successfully"));
    println!("  {}", formatter.format_warning("This operation is irreversible"));
    println!("  {}", formatter.format_error("Failed to push: remote rejected"));
    println!();

    // Demo 6: Diff Output
    println!("--- 6. DIFF OUTPUT ---");
    let diff_hunks = vec![DiffHunk {
        file: "src/main.rs".to_string(),
        added: 3,
        removed: 2,
        lines: vec![
            DiffLine::Context("fn main() {".to_string()),
            DiffLine::Removed("    println!(\"old\");".to_string()),
            DiffLine::Added("    println!(\"Hello, World!\");".to_string()),
            DiffLine::Added("    println!(\"New features!\");".to_string()),
            DiffLine::Context("    process();".to_string()),
            DiffLine::Removed("    deprecated_call();".to_string()),
            DiffLine::Context("}".to_string()),
        ],
    }];
    println!("{}\n", formatter.format_diff(&diff_hunks));

    // Demo 7: Merge Conflict
    println!("--- 7. MERGE CONFLICT ---");
    let conflict = formatter.format_merge_conflict(
        "src/config.rs",
        "let timeout = 5000;  // Production timeout",
        "let timeout = 30000; // Testing timeout",
    );
    println!("{}\n", conflict);

    // Demo 8: Empty status
    println!("--- 8. CLEAN WORKING DIRECTORY ---");
    println!("{}\n", formatter.format_status("main", &[]));

    // Demo 9: ASCII-only mode
    println!("--- 9. ASCII FALLBACK MODE ---");
    let ascii_formatter = UnicodeFormatter::new(false, false);
    let simple_commits = vec![CommitInfo {
        hash: "abc1234".to_string(),
        author: "Dev".to_string(),
        date: "2025-12-29".to_string(),
        message: "Test commit".to_string(),
        is_head: true,
        branch: Some("ascii-test".to_string()),
    }];
    println!("{}\n", ascii_formatter.format_log(&simple_commits));

    println!("=== End of Demo ===\n");
}
