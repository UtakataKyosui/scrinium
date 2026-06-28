use scrinium::validate::{ValidationResult, validate_path};
use std::fs;
use tempfile::TempDir;

fn full_okf(title: &str) -> String {
    format!(
        "---\nid: \"test-uuid\"\ntype: \"Concept\"\ntitle: \"{title}\"\ndescription: \"desc\"\ntimestamp: \"2026-01-01T00:00:00Z\"\n---\n\n# {title}\n"
    )
}

#[test]
fn validate_directory_skips_dotdirs() {
    let dir = TempDir::new().unwrap();
    // Valid file in root
    fs::write(dir.path().join("valid.md"), full_okf("Valid")).unwrap();
    // File inside a dot-directory — should be skipped
    let hidden = dir.path().join(".hidden");
    fs::create_dir(&hidden).unwrap();
    fs::write(hidden.join("secret.md"), "# No frontmatter").unwrap();

    let results = validate_path(dir.path()).unwrap();
    assert_eq!(results.len(), 1, "dotdir .md must not appear in results");
    assert!(matches!(results[0], ValidationResult::Pass(_)));
}

#[test]
fn validate_directory_all_pass() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.md"), full_okf("A")).unwrap();
    fs::write(dir.path().join("b.md"), full_okf("B")).unwrap();

    let results = validate_path(dir.path()).unwrap();
    assert_eq!(results.len(), 2);
    for r in &results {
        assert!(matches!(r, ValidationResult::Pass(_)));
    }
}

#[test]
fn validate_single_file_no_fm_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bare.md");
    fs::write(&path, "# Just Markdown\n").unwrap();

    let results = validate_path(&path).unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], ValidationResult::Fail(_, _)));
}
