use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::path::{Path, PathBuf};

use super::theme;
use scrinium::is_markdown;

pub struct FileBrowser {
    pub entries: Vec<PathBuf>,
    pub list_state: ListState,
    pub dir: PathBuf,
}

impl FileBrowser {
    pub fn new(dir: PathBuf) -> anyhow::Result<Self> {
        let mut browser = Self {
            entries: Vec::new(),
            list_state: ListState::default(),
            dir,
        };
        browser.refresh()?;
        Ok(browser)
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| is_markdown(p))
            .collect();
        entries.sort();
        self.entries = entries;
        if self.list_state.selected().is_none() && !self.entries.is_empty() {
            self.list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    pub fn select_by_path(&mut self, path: &Path) {
        if let Some(pos) = self.entries.iter().position(|e| e == path) {
            self.list_state.select(Some(pos));
        }
    }

    pub fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 {
            self.list_state.select(Some(i - 1));
        }
    }

    pub fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.entries.len() {
            self.list_state.select(Some(i + 1));
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" FILES ")
            .border_style(if focused {
                theme::border_focused()
            } else {
                theme::border_unfocused()
            })
            .style(theme::base());

        let selected_idx = self.list_state.selected().unwrap_or(usize::MAX);
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                let (prefix, style): (&str, Style) = if i == selected_idx && focused {
                    ("> ", theme::selected_file())
                } else if i == selected_idx {
                    ("> ", Style::default().fg(ratatui::style::Color::White))
                } else {
                    ("  ", Style::default().fg(theme::DIM_FG))
                };
                ListItem::new(format!("{prefix}{name}")).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_stateful_widget(list, area, &mut self.list_state);
    }
}
