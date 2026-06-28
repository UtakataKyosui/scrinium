use scrinium::bundle::Bundle;
use std::fs;
use tempfile::TempDir;

fn concept_md(title: &str) -> String {
    format!(
        "---\nid: \"uuid\"\ntype: \"Concept\"\ntitle: \"{title}\"\ndescription: \"d\"\ntimestamp: \"2026-01-01T00:00:00Z\"\n---\n\n# {title}\n"
    )
}

#[test]
fn bundle_loads_concept_docs() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.md"), concept_md("A")).unwrap();
    fs::write(dir.path().join("b.md"), concept_md("B")).unwrap();
    // index.md and log.md must be excluded from concept_docs()
    fs::write(
        dir.path().join("index.md"),
        "---\ntype: \"Index\"\n---\n\nIndex\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("log.md"),
        "---\ntype: \"ChangeLog\"\n---\n\nLog\n",
    )
    .unwrap();

    let bundle = Bundle::load(dir.path()).unwrap();
    let concepts: Vec<_> = bundle.concept_docs().collect();
    assert_eq!(concepts.len(), 2);
    let titles: Vec<_> = concepts
        .iter()
        .map(|d| d.frontmatter.title.as_deref().unwrap_or(""))
        .collect();
    assert!(titles.contains(&"A"));
    assert!(titles.contains(&"B"));
}

#[test]
fn bundle_skips_unparseable_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("good.md"), concept_md("Good")).unwrap();
    // File without frontmatter — bundle should log a warning and continue
    fs::write(dir.path().join("bad.md"), "# No frontmatter\n").unwrap();

    let bundle = Bundle::load(dir.path()).unwrap();
    // Only the parseable file ends up in documents
    assert_eq!(bundle.documents.len(), 1);
}
