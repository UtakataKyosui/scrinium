use crate::document::Frontmatter;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use tui_textarea::TextArea;

use super::theme;

const FIELDS: &[&str] = &[
    "id",
    "type",
    "title",
    "description",
    "resource",
    "tags",
    "timestamp",
];

pub struct FrontmatterEditor {
    pub textareas: Vec<TextArea<'static>>,
    pub active_field: usize,
}

impl FrontmatterEditor {
    pub fn new() -> Self {
        let textareas = FIELDS.iter().map(|_| make_textarea("")).collect();
        Self {
            textareas,
            active_field: 0,
        }
    }

    pub fn load(&mut self, fm: &Frontmatter) {
        let values = [
            fm.id.clone().unwrap_or_default(),
            fm.doc_type.clone(),
            fm.title.clone().unwrap_or_default(),
            fm.description.clone().unwrap_or_default(),
            fm.resource.clone().unwrap_or_default(),
            fm.tags
                .as_ref()
                .map(|t| t.join(", "))
                .unwrap_or_default(),
            fm.timestamp.clone().unwrap_or_default(),
        ];
        for (i, val) in values.iter().enumerate() {
            self.textareas[i] = make_textarea(val);
        }
        self.active_field = 0;
    }

    pub fn to_frontmatter(&self) -> Frontmatter {
        let get = |i: usize| -> String {
            self.textareas[i]
                .lines()
                .first()
                .cloned()
                .unwrap_or_default()
                .trim()
                .to_string()
        };
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
                    Some(raw.split(',').map(|s| s.trim().to_string()).collect())
                }
            },
            timestamp: opt(get(6)),
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Down | KeyCode::Enter => {
                    if self.active_field + 1 < FIELDS.len() {
                        self.active_field += 1;
                    }
                    return true;
                }
                KeyCode::Up => {
                    if self.active_field > 0 {
                        self.active_field -= 1;
                    }
                    return true;
                }
                _ => {
                    let input = tui_textarea::Input::from(event.clone());
                    self.textareas[self.active_field].input(input);
                    return true;
                }
            }
        }
        false
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        use ratatui::widgets::{Block, Borders};

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

        let constraints: Vec<Constraint> = FIELDS
            .iter()
            .map(|_| Constraint::Length(1))
            .chain(std::iter::once(Constraint::Min(0)))
            .collect();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        for (i, label) in FIELDS.iter().enumerate() {
            let is_active = focused && i == self.active_field;
            let row = rows[i];

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
                self.textareas[i]
                    .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
                self.textareas[i].set_style(theme::base());
                f.render_widget(&self.textareas[i], cols[1]);
            } else {
                self.textareas[i].set_cursor_style(Style::default());
                let val = self.textareas[i]
                    .lines()
                    .first()
                    .cloned()
                    .unwrap_or_default();
                f.render_widget(
                    Paragraph::new(val).style(Style::default().fg(theme::FG)),
                    cols[1],
                );
            }
        }
    }
}

fn make_textarea(value: &str) -> TextArea<'static> {
    let mut ta = TextArea::new(vec![value.to_string()]);
    ta.set_cursor_line_style(Style::default());
    ta.set_cursor_style(Style::default());
    ta.set_style(theme::base());
    ta
}
