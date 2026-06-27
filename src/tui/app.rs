use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use std::{collections::BTreeMap, path::PathBuf};

use crate::document::Document;

use super::{
    browser::FileBrowser, editor::MarkdownEditor, effects::Effects,
    frontmatter::FrontmatterEditor,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Browse,
    EditFrontmatter,
    EditMarkdown,
    HelpPopup,
}

pub struct App {
    pub mode: AppMode,
    pub browser: FileBrowser,
    pub fm_editor: FrontmatterEditor,
    pub md_editor: MarkdownEditor,
    pub current_file: Option<PathBuf>,
    pub is_dirty: bool,
    pub status_message: String,
    pub status_is_error: bool,
    pub should_quit: bool,
    pub effects: Effects,
    pub last_full_area: Rect,
    pub last_right_area: Rect,
    pub last_status_area: Rect,
}

impl App {
    pub fn new(initial_file: Option<PathBuf>) -> Result<Self> {
        let dir = initial_file
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let mut app = Self {
            mode: AppMode::Browse,
            browser: FileBrowser::new(dir)?,
            fm_editor: FrontmatterEditor::new(),
            md_editor: MarkdownEditor::new(),
            current_file: None,
            is_dirty: false,
            status_message: String::new(),
            status_is_error: false,
            should_quit: false,
            effects: Effects::new(),
            last_full_area: Rect::default(),
            last_right_area: Rect::default(),
            last_status_area: Rect::default(),
        };

        if let Some(path) = initial_file {
            app.open_file(path)?;
        }
        Ok(app)
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<()> {
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", path.display(), e))?;

        let (frontmatter, body, extra) = match Document::from_path(&path) {
            Ok(doc) => {
                let extra = extract_extra_yaml_fields(&raw);
                (doc.frontmatter, doc.body, extra)
            }
            Err(_) => {
                // ファイルにフロントマターがない場合は全内容を body として扱う
                let title = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let fm = crate::document::Frontmatter {
                    id: None,
                    doc_type: String::new(),
                    title,
                    description: None,
                    resource: None,
                    tags: None,
                    timestamp: None,
                };
                (fm, raw, BTreeMap::new())
            }
        };

        self.fm_editor.load(&frontmatter, extra);
        self.md_editor.load(&body);
        self.browser.select_by_path(&path);
        self.current_file = Some(path);
        self.is_dirty = false;
        self.status_message = "File opened".to_string();
        self.status_is_error = false;
        self.effects.trigger_file_open(self.last_right_area);
        self.mode = AppMode::EditMarkdown;
        Ok(())
    }

    pub fn save_file(&mut self) -> Result<()> {
        let Some(path) = self.current_file.clone() else {
            self.status_message = "No file is open".to_string();
            self.status_is_error = true;
            return Ok(());
        };

        if self.fm_editor.get_doc_type().trim().is_empty() {
            self.status_message = "Error: 'type' field is required".to_string();
            self.status_is_error = true;
            return Ok(());
        }

        // Auto-update timestamp to current UTC time
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.fm_editor.set_timestamp(&now);

        let yaml = self.fm_editor.to_yaml_string()?;
        let body = self.md_editor.content();
        let content = format!("---\n{yaml}---\n\n{body}");
        std::fs::write(&path, content)?;

        self.is_dirty = false;
        self.status_message = format!("Saved at {now}");
        self.status_is_error = false;
        self.effects.trigger_save_flash(self.last_status_area);
        Ok(())
    }

    pub fn focus_next(&mut self) {
        let next = match self.mode {
            AppMode::Browse => AppMode::EditFrontmatter,
            AppMode::EditFrontmatter => AppMode::EditMarkdown,
            AppMode::EditMarkdown => AppMode::Browse,
            AppMode::HelpPopup => return,
        };
        self.effects.trigger_focus_switch(self.last_full_area);
        self.mode = next;
    }

    pub fn focus_prev(&mut self) {
        let prev = match self.mode {
            AppMode::Browse => AppMode::EditMarkdown,
            AppMode::EditFrontmatter => AppMode::Browse,
            AppMode::EditMarkdown => AppMode::EditFrontmatter,
            AppMode::HelpPopup => return,
        };
        self.effects.trigger_focus_switch(self.last_full_area);
        self.mode = prev;
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Key(key) = event {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => return self.save_file(),
                (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                    self.should_quit = true;
                    return Ok(());
                }
                (_, KeyCode::F(1)) => {
                    self.mode = if self.mode == AppMode::HelpPopup {
                        AppMode::Browse
                    } else {
                        AppMode::HelpPopup
                    };
                    return Ok(());
                }
                (_, KeyCode::Tab) => {
                    self.focus_next();
                    return Ok(());
                }
                (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                    self.focus_prev();
                    return Ok(());
                }
                _ => {}
            }

            match self.mode.clone() {
                AppMode::Browse => self.handle_browse_key(key)?,
                AppMode::EditFrontmatter => {
                    if self.fm_editor.handle_input(event) {
                        self.is_dirty = true;
                        self.status_message.clear();
                    }
                }
                AppMode::EditMarkdown => {
                    if self.md_editor.handle_input(event) {
                        self.is_dirty = true;
                        self.status_message.clear();
                    }
                }
                AppMode::HelpPopup => {
                    self.mode = AppMode::Browse;
                }
            }
        }
        Ok(())
    }

    fn handle_browse_key(&mut self, key: &crossterm::event::KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.browser.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.browser.move_down(),
            KeyCode::Enter => {
                if let Some(path) = self.browser.selected_path().cloned() {
                    if let Err(e) = self.open_file(path) {
                        self.status_message = format!("Error: {e}");
                        self.status_is_error = true;
                    }
                }
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Extract YAML fields that are not in STANDARD_KEYS from raw file content.
fn extract_extra_yaml_fields(content: &str) -> BTreeMap<String, serde_yaml::Value> {
    const STANDARD_KEYS: &[&str] = &[
        "id", "type", "title", "description", "resource", "tags", "timestamp",
    ];

    let mut extra = BTreeMap::new();

    // Find frontmatter block between --- delimiters
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return extra;
    }
    let after_open = &trimmed[3..];
    let end = after_open.find("\n---").unwrap_or(0);
    if end == 0 {
        return extra;
    }
    let yaml_block = &after_open[..end];

    if let Ok(serde_yaml::Value::Mapping(map)) = serde_yaml::from_str::<serde_yaml::Value>(yaml_block) {
        for (k, v) in map {
            if let serde_yaml::Value::String(key) = k {
                if !STANDARD_KEYS.contains(&key.as_str()) {
                    extra.insert(key, v);
                }
            }
        }
    }

    extra
}
