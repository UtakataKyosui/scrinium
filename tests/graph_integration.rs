use scrinium::bundle::Bundle;
use scrinium::graph::GraphData;
use std::fs;
use tempfile::TempDir;

fn concept_with_link(title: &str, id: &str, link_to: Option<&str>) -> String {
    let body = match link_to {
        Some(target) => format!("\n# {title}\n\nSee [{target}]({target}.md).\n"),
        None => format!("\n# {title}\n\nNo links.\n"),
    };
    format!(
        "---\nid: \"{id}\"\ntype: \"Concept\"\ntitle: \"{title}\"\ndescription: \"d\"\ntimestamp: \"2026-01-01T00:00:00Z\"\n---\n{body}"
    )
}

#[test]
fn graph_builds_edges_from_md_links() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("a.md"),
        concept_with_link("A", "id-a", Some("b")),
    )
    .unwrap();
    fs::write(
        dir.path().join("b.md"),
        concept_with_link("B", "id-b", None),
    )
    .unwrap();

    let bundle = Bundle::load(dir.path()).unwrap();
    let graph = GraphData::from_bundle(&bundle);

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].source, "id-a");
    assert_eq!(graph.edges[0].target, "id-b");
}

#[test]
fn graph_to_json_roundtrip() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("x.md"),
        concept_with_link("X", "id-x", None),
    )
    .unwrap();

    let bundle = Bundle::load(dir.path()).unwrap();
    let graph = GraphData::from_bundle(&bundle);
    let json = graph.to_json().unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let nodes = parsed["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["id"], "id-x");
    assert_eq!(nodes[0]["title"], "X");
}
