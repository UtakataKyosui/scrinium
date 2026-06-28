pub mod bundle;
pub mod document;
pub mod graph;
pub mod validate;

/// Returns `true` if a WalkDir entry should be visited.
/// Skips hidden directories (`.`-prefixed) and build artifact directories (`target`).
pub fn should_visit_entry(entry: &walkdir::DirEntry) -> bool {
    if entry.depth() == 0 {
        return true;
    }
    entry
        .file_name()
        .to_str()
        .map(|s| !s.starts_with('.') && s != "target")
        .unwrap_or(true)
}

/// Returns `true` if `path` has a `.md` extension.
pub fn is_markdown(path: &std::path::Path) -> bool {
    path.extension().is_some_and(|ext| ext == "md")
}

/// Returns an iterator over all Markdown files reachable from `dir`.
///
/// Skips hidden directories and `target/` via [`should_visit_entry`], does not
/// follow symbolic links, and yields only entries with a `.md` extension.
pub fn markdown_files(dir: &std::path::Path) -> impl Iterator<Item = walkdir::DirEntry> {
    walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(should_visit_entry)
        .filter_map(|e| e.ok())
        .filter(|e| is_markdown(e.path()))
}
