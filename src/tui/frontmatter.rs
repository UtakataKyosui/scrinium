use crate::document::Frontmatter;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::BTreeMap;
use tui_textarea::TextArea;

use super::theme;

const STANDARD_KEYS: &[&str] = &[
    "id", "type", "title", "description", "resource", "tags", "timestamp",
];

// Display labels for standard fields (right-aligned in 12 chars)
const FIELD_LABELS: &[&str] = &[
    "id",
    "type *",      // * = required
    "title",
    "description",
    "resource",
    "tags",        // hint shown in hint bar when focused
    "timestamp",   // auto-updated on save
];

pub struct FrontmatterEditor {
    pub textareas: Vec<TextArea<'static>>,             // 7 standard fields
    pub extra_fields: Vec<(String, TextArea<'static>)>, // arbitrary extra fields
    pub active_field: usize, // 0..7 = standard, 7+ = extra
    new_key_input: Option<TextArea<'static>>,           // when Ctrl+N is pressed
}

impl FrontmatterEditor {
    pub fn new() -> Self {
        let textareas = STANDARD_KEYS.iter().map(|_| make_textarea("")).collect();
        Self {
            textareas,
            extra_fields: Vec::new(),
            active_field: 0,
            new_key_input: None,
        }
    }

    /// Load from parsed frontmatter + arbitrary extra YAML fields.
    pub fn load(&mut self, fm: &Frontmatter, extra: BTreeMap<String, serde_yaml::Value>) {
        let values = [
            fm.id.clone().unwrap_or_default(),
            fm.doc_type.clone(),
            fm.title.clone().unwrap_or_default(),
            fm.description.clone().unwrap_or_default(),
            fm.resource.clone().unwrap_or_default(),
            fm.tags.as_ref().map(|t| t.join(", ")).unwrap_or_default(),
            fm.timestamp.clone().unwrap_or_default(),
        ];
        for (i, val) in values.iter().enumerate() {
            self.textareas[i] = make_textarea(val);
        }

        self.extra_fields = extra
            .into_iter()
            .map(|(k, v)| {
                let display = value_to_display(&v);
                (k, make_textarea(&display))
            })
            .collect();

        self.active_field = 0;
        self.new_key_input = None;
    }

    /// Overwrite the timestamp textarea (called by save_file before serializing).
    pub fn set_timestamp(&mut self, ts: &str) {
        self.textareas[6] = make_textarea(ts);
    }

    /// Return the "type" field value for validation.
    pub fn get_doc_type(&self) -> String {
        ta_val(&self.textareas[1])
    }

    /// Serialize all fields (standard + extra) to a YAML string suitable for frontmatter.
    pub fn to_yaml_string(&self) -> anyhow::Result<String> {
        let fm = self.to_frontmatter_standard();
        let mut yaml_val = serde_yaml::to_value(&fm)?;

        if let serde_yaml::Value::Mapping(ref mut map) = yaml_val {
            for (key, ta) in &self.extra_fields {
                let val_str = ta_val(ta);
                // Try to parse as a proper YAML value; fall back to plain string
                let val = serde_yaml::from_str::<serde_yaml::Value>(&val_str)
                    .unwrap_or_else(|_| serde_yaml::Value::String(val_str));
                map.insert(serde_yaml::Value::String(key.clone()), val);
            }
        }

        Ok(serde_yaml::to_string(&yaml_val)?)
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            // ── new_key_input mode ───────────────────────────────────────
            if let Some(ref mut key_ta) = self.new_key_input {
                match key.code {
                    KeyCode::Enter => {
                        let new_key = ta_val(key_ta);
                        self.new_key_input = None;
                        if !new_key.is_empty() && !STANDARD_KEYS.contains(&new_key.as_str()) {
                            self.extra_fields.push((new_key, make_textarea("")));
                            self.active_field = STANDARD_KEYS.len() + self.extra_fields.len() - 1;
                        }
                    }
                    KeyCode::Esc => {
                        self.new_key_input = None;
                    }
                    _ => {
                        key_ta.input(tui_textarea::Input::from(event.clone()));
                    }
                }
                return true;
            }

            let total = STANDARD_KEYS.len() + self.extra_fields.len();

            match (key.modifiers, key.code) {
                // Add new custom field
                (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                    self.new_key_input = Some(make_textarea(""));
                    return true;
                }
                // Field navigation
                (_, KeyCode::Down) | (_, KeyCode::Enter) => {
                    if self.active_field + 1 < total {
                        self.active_field += 1;
                    }
                    return true;
                }
                (_, KeyCode::Up) => {
                    if self.active_field > 0 {
                        self.active_field -= 1;
                    }
                    return true;
                }
                // Text input for active field
                _ => {
                    if self.active_field < STANDARD_KEYS.len() {
                        self.textareas[self.active_field]
                            .input(tui_textarea::Input::from(event.clone()));
                    } else {
                        let ei = self.active_field - STANDARD_KEYS.len();
                        if let Some((_, ta)) = self.extra_fields.get_mut(ei) {
                            ta.input(tui_textarea::Input::from(event.clone()));
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let outer = Block::default()
            .borders(Borders::ALL)
            .title(" FRONTMATTER ")
            .border_style(if focused {
                theme::border_focused()
            } else {
                theme::border_unfocused()
            })
            .style(theme::base());

        let inner = outer.inner(area);
        f.render_widget(outer, area);

        let has_extras = !self.extra_fields.is_empty();
        let has_new_key = self.new_key_input.is_some();

        // Count content rows
        let mut n_content = STANDARD_KEYS.len();
        if has_extras {
            n_content += 1 + self.extra_fields.len(); // separator + fields
        }
        if has_new_key {
            n_content += 1;
        }

        let mut constraints: Vec<Constraint> = (0..n_content).map(|_| Constraint::Length(1)).collect();
        constraints.push(Constraint::Min(0));    // spacer
        constraints.push(Constraint::Length(1)); // hint bar

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let hint_row = rows[rows.len() - 1];

        // ── Standard fields ──────────────────────────────────────────────
        for i in 0..STANDARD_KEYS.len() {
            let is_active = focused && i == self.active_field;
            render_field_row(f, &mut self.textareas[i], FIELD_LABELS[i], rows[i], is_active);
        }

        let mut row_cursor = STANDARD_KEYS.len();

        // ── Extra fields ──────────────────────────────────────────────────
        if has_extras {
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("── ", Style::default().fg(theme::BRAND_BLUE)),
                    Span::styled("拡張フィールド", theme::label_inactive()),
                    Span::styled(" ──", Style::default().fg(theme::BRAND_BLUE)),
                ])),
                rows[row_cursor],
            );
            row_cursor += 1;

            for ei in 0..self.extra_fields.len() {
                let global_i = STANDARD_KEYS.len() + ei;
                let is_active = focused && global_i == self.active_field;
                let key = self.extra_fields[ei].0.clone();
                let ta = &mut self.extra_fields[ei].1;
                render_field_row(f, ta, &key, rows[row_cursor], is_active);
                row_cursor += 1;
            }
        }

        // ── New key input ─────────────────────────────────────────────────
        if let Some(ref mut key_ta) = self.new_key_input {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(14), Constraint::Min(0)])
                .split(rows[row_cursor]);

            f.render_widget(
                Paragraph::new("新フィールド: ").style(theme::label_active()),
                cols[0],
            );
            key_ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            key_ta.set_style(theme::base());
            f.render_widget(&*key_ta, cols[1]);
        }

        // ── Hint bar ─────────────────────────────────────────────────────
        let hint = if self.new_key_input.is_some() {
            Line::from(vec![
                Span::styled(" Enter", theme::label_active()),
                Span::raw(": 確定  "),
                Span::styled("Esc", theme::label_active()),
                Span::raw(": キャンセル"),
            ])
        } else if focused {
            match self.active_field {
                5 => Line::from(vec![
                    Span::styled(" tags", theme::label_active()),
                    Span::raw(": カンマ区切りで入力 (例: rust, tui, editor)"),
                ]),
                6 => Line::from(vec![
                    Span::styled(" timestamp", theme::label_active()),
                    Span::raw(": 保存時に自動更新されます"),
                ]),
                _ => Line::from(vec![
                    Span::styled(" ↑↓/Enter", theme::label_active()),
                    Span::raw(": 移動  "),
                    Span::styled("Ctrl+N", theme::label_active()),
                    Span::raw(": 拡張フィールド追加"),
                ]),
            }
        } else {
            Line::from("")
        };

        f.render_widget(
            Paragraph::new(hint).style(Style::default().bg(theme::BG)),
            hint_row,
        );
    }

    // Internal: build Frontmatter from standard textarea values only
    fn to_frontmatter_standard(&self) -> Frontmatter {
        let get = |i: usize| ta_val(&self.textareas[i]);
        let opt = |s: String| if s.is_empty() { None } else { Some(s) };

        Frontmatter {
            id: opt(get(0)),
            doc_type: get(1),
            title: opt(get(2)),
            description: opt(get(3)),
            resource: opt(get(4)),
            tags: {
                let raw = get(5);
                if raw.is_empty() {
                    None
                } else {
                    Some(
                        raw.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect(),
                    )
                }
            },
            timestamp: opt(get(6)),
        }
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn ta_val(ta: &TextArea<'static>) -> String {
    ta.lines().first().cloned().unwrap_or_default().trim().to_string()
}

fn value_to_display(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => String::new(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

fn make_textarea(value: &str) -> TextArea<'static> {
    let mut ta = TextArea::new(vec![value.to_string()]);
    ta.set_cursor_line_style(Style::default());
    ta.set_cursor_style(Style::default());
    ta.set_style(theme::base());
    ta
}

fn render_field_row(
    f: &mut Frame,
    ta: &mut TextArea<'static>,
    label: &str,
    row: Rect,
    is_active: bool,
) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(0)])
        .split(row);

    let lbl_style = if is_active {
        theme::label_active()
    } else {
        theme::label_inactive()
    };
    f.render_widget(
        Paragraph::new(format!("{:>12}: ", label)).style(lbl_style),
        cols[0],
    );

    if is_active {
        ta.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        ta.set_style(theme::base());
        f.render_widget(&*ta, cols[1]);
    } else {
        ta.set_cursor_style(Style::default());
        let val = ta.lines().first().cloned().unwrap_or_default();
        f.render_widget(
            Paragraph::new(val).style(Style::default().fg(theme::FG)),
            cols[1],
        );
    }
}
