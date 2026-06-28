use crate::bundle::Bundle;
use crate::document::Document;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub doc_type: String,
    pub path: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl GraphData {
    pub fn from_bundle(bundle: &Bundle) -> Self {
        let docs: Vec<&Document> = bundle.concept_docs().collect();

        // Map path -> node id
        let mut path_to_id: HashMap<String, String> = HashMap::new();
        let mut nodes: Vec<GraphNode> = Vec::new();

        for (i, doc) in docs.iter().enumerate() {
            let id = doc
                .frontmatter
                .id
                .clone()
                .unwrap_or_else(|| format!("node-{i}"));
            // Normalize path so resolve_link lookups match (removes leading ./)
            let path_str = normalize_path(&doc.path);
            path_to_id.insert(path_str.clone(), id.clone());

            let angle = 2.0 * std::f64::consts::PI * i as f64 / docs.len().max(1) as f64;
            let r = (WIDTH.min(HEIGHT) * 0.38).max(1.0);
            nodes.push(GraphNode {
                id,
                title: doc.display_title(),
                doc_type: doc.frontmatter.doc_type.clone(),
                path: path_str,
                x: WIDTH / 2.0 + r * angle.cos(),
                y: HEIGHT / 2.0 + r * angle.sin(),
            });
        }

        // Build edges from Markdown links
        let mut edges: Vec<GraphEdge> = Vec::new();
        for doc in &docs {
            let src_path = normalize_path(&doc.path);
            let source = match path_to_id.get(&src_path) {
                Some(id) => id.clone(),
                None => continue,
            };
            for link in &doc.md_links {
                let resolved = resolve_link(&doc.path, link, &bundle.root);
                if let Some(target) = resolved.and_then(|p| path_to_id.get(&p).cloned()) {
                    edges.push(GraphEdge {
                        source: source.clone(),
                        target,
                    });
                }
            }
        }

        let mut data = Self { nodes, edges };
        data.layout();
        data
    }

    fn layout(&mut self) {
        let n = self.nodes.len();
        if n < 2 {
            return;
        }

        // Node id -> index for fast edge lookup
        let idx_map: HashMap<String, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.id.clone(), i))
            .collect();

        let k = (WIDTH * HEIGHT / n as f64).sqrt();
        let mut temp = WIDTH * 0.1;
        let mut disp: Vec<(f64, f64)> = vec![(0.0, 0.0); n];

        for _ in 0..150 {
            // Repulsive forces (needs dual index: disp[i] and nodes[i] vs nodes[j])
            #[allow(clippy::needless_range_loop)]
            for i in 0..n {
                disp[i] = (0.0, 0.0);
                for j in 0..n {
                    if i == j {
                        continue;
                    }
                    let dx = self.nodes[i].x - self.nodes[j].x;
                    let dy = self.nodes[i].y - self.nodes[j].y;
                    let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                    let f = k * k / dist;
                    disp[i].0 += dx / dist * f;
                    disp[i].1 += dy / dist * f;
                }
            }

            // Attractive forces along edges
            for edge in &self.edges {
                let (Some(&si), Some(&ti)) = (idx_map.get(&edge.source), idx_map.get(&edge.target))
                else {
                    continue;
                };
                let dx = self.nodes[si].x - self.nodes[ti].x;
                let dy = self.nodes[si].y - self.nodes[ti].y;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let f = dist * dist / k;
                disp[si].0 -= dx / dist * f;
                disp[si].1 -= dy / dist * f;
                disp[ti].0 += dx / dist * f;
                disp[ti].1 += dy / dist * f;
            }

            // Apply displacements with temperature-capped step
            for (node, d) in self.nodes.iter_mut().zip(disp.iter()) {
                let mag = (d.0 * d.0 + d.1 * d.1).sqrt().max(0.01);
                let scale = mag.min(temp) / mag;
                node.x = (node.x + d.0 * scale).clamp(40.0, WIDTH - 40.0);
                node.y = (node.y + d.1 * scale).clamp(40.0, HEIGHT - 40.0);
            }
            temp *= 0.95;
        }
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_svg(&self) -> String {
        const PALETTE: &[&str] = &[
            "#e94560", "#4a9eff", "#533483", "#05c46b", "#ffd460", "#ff8a5b", "#a8e6cf", "#d4a5a5",
        ];

        let mut type_color: HashMap<&str, &str> = HashMap::new();
        let mut ci = 0usize;
        for node in &self.nodes {
            if !type_color.contains_key(node.doc_type.as_str()) {
                type_color.insert(&node.doc_type, PALETTE[ci % PALETTE.len()]);
                ci += 1;
            }
        }

        let node_map: HashMap<&str, &GraphNode> =
            self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let w = WIDTH as u32;
        let h = HEIGHT as u32;
        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
<rect width="100%" height="100%" fill="#1a1a2e"/>
<defs>
  <marker id="arr" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
    <polygon points="0 0,8 3,0 6" fill="#4a9eff" opacity="0.5"/>
  </marker>
</defs>
"##
        );

        // Edges
        for edge in &self.edges {
            if let (Some(s), Some(t)) = (
                node_map.get(edge.source.as_str()),
                node_map.get(edge.target.as_str()),
            ) {
                svg.push_str(&format!(
                    "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#4a9eff\" stroke-width=\"1.2\" opacity=\"0.4\" marker-end=\"url(#arr)\"/>\n",
                    s.x, s.y, t.x, t.y
                ));
            }
        }

        // Nodes
        for node in &self.nodes {
            let color = type_color
                .get(node.doc_type.as_str())
                .copied()
                .unwrap_or("#888");
            let label = truncate(&node.title, 18);
            let type_label = escape_xml(&node.doc_type);
            let body_label = escape_xml(&label);
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"20\" fill=\"{color}\" stroke=\"#fff\" stroke-width=\"1.5\" opacity=\"0.9\"/>\n\
                 <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"9\" fill=\"{color}\" opacity=\"0.75\">{type_label}</text>\n\
                 <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"10\" fill=\"#fff\">{body_label}</text>\n",
                node.x, node.y,
                node.x, node.y - 27.0,
                node.x, node.y + 33.0,
            ));
        }

        svg.push_str("</svg>");
        svg
    }

    pub fn to_png(&self, output: &Path) -> Result<()> {
        use tiny_skia::*;

        const PALETTE: &[[u8; 3]] = &[
            [233, 69, 96],
            [74, 158, 255],
            [83, 52, 131],
            [5, 196, 107],
            [255, 212, 96],
            [255, 138, 91],
            [168, 230, 207],
            [212, 165, 165],
        ];

        let w = WIDTH as u32;
        let h = HEIGHT as u32;
        let mut pixmap =
            Pixmap::new(w, h).ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;
        pixmap.fill(Color::from_rgba8(26, 26, 46, 255));

        let node_map: HashMap<&str, &GraphNode> =
            self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        // Edges
        for edge in &self.edges {
            if let (Some(s), Some(t)) = (
                node_map.get(edge.source.as_str()),
                node_map.get(edge.target.as_str()),
            ) {
                let mut pb = PathBuilder::new();
                pb.move_to(s.x as f32, s.y as f32);
                pb.line_to(t.x as f32, t.y as f32);
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(74, 158, 255, 100);
                    pixmap.stroke_path(
                        &path,
                        &paint,
                        &Stroke {
                            width: 1.5,
                            ..Default::default()
                        },
                        Transform::identity(),
                        None,
                    );
                }
            }
        }

        // Assign type -> color index
        let mut type_idx: HashMap<&str, usize> = HashMap::new();
        let mut ci = 0usize;
        for node in &self.nodes {
            if !type_idx.contains_key(node.doc_type.as_str()) {
                type_idx.insert(&node.doc_type, ci % PALETTE.len());
                ci += 1;
            }
        }

        // Nodes
        for node in &self.nodes {
            let [r, g, b] = PALETTE[type_idx.get(node.doc_type.as_str()).copied().unwrap_or(0)];
            if let Some(rect) =
                Rect::from_xywh(node.x as f32 - 20.0, node.y as f32 - 20.0, 40.0, 40.0)
            {
                let mut pb = PathBuilder::new();
                pb.push_oval(rect);
                if let Some(path) = pb.finish() {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(r, g, b, 230);
                    pixmap.fill_path(
                        &path,
                        &paint,
                        FillRule::EvenOdd,
                        Transform::identity(),
                        None,
                    );

                    paint.set_color_rgba8(255, 255, 255, 180);
                    pixmap.stroke_path(
                        &path,
                        &paint,
                        &Stroke {
                            width: 1.5,
                            ..Default::default()
                        },
                        Transform::identity(),
                        None,
                    );
                }
            }
        }

        pixmap
            .save_png(output)
            .map_err(|e| anyhow::anyhow!("PNG save error: {e}"))?;
        Ok(())
    }
}

fn resolve_link(from: &Path, link: &str, bundle_root: &Path) -> Option<String> {
    let link = link.split('#').next()?;
    if link.starts_with("http://") || link.starts_with("https://") {
        return None;
    }
    let target = if let Some(stripped) = link.strip_prefix('/') {
        bundle_root.join(stripped)
    } else {
        from.parent()?.join(link)
    };
    Some(normalize_path(&target))
}

fn normalize_path(path: &Path) -> String {
    let mut result = std::path::PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            c => result.push(c),
        }
    }
    result.to_string_lossy().to_string()
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let mut result: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn normalize_path_resolves_dotdot() {
        let p = PathBuf::from("a/b/../c");
        assert_eq!(normalize_path(&p), "a/c");
    }

    #[test]
    fn normalize_path_removes_curdir() {
        let p = PathBuf::from("a/./b");
        assert_eq!(normalize_path(&p), "a/b");
    }

    #[test]
    fn truncate_short_unchanged() {
        assert_eq!(truncate("hello", 18), "hello");
    }

    #[test]
    fn truncate_long_adds_ellipsis() {
        let s = "a".repeat(20);
        let result = truncate(&s, 18);
        assert!(result.ends_with('…'));
        assert_eq!(result.chars().count(), 19); // 18 chars + ellipsis
    }

    #[test]
    fn escape_xml_special_chars() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn to_json_has_nodes_and_edges_keys() {
        let bundle = Bundle {
            root: PathBuf::from("."),
            documents: vec![],
        };
        let data = GraphData::from_bundle(&bundle);
        let json = data.to_json().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("nodes").is_some());
        assert!(v.get("edges").is_some());
    }
}
