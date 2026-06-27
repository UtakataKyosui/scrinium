use crossterm::event::Event;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
    Frame,
};
use tui_textarea::TextArea;

use super::theme;

pub struct MarkdownEditor {
    pub textarea: TextArea<'static>,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        let mut ta = TextArea::default();
        configure(&mut ta);
        Self { textarea: ta }
    }

    pub fn load(&mut self, content: &str) {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        self.textarea = TextArea::new(lines);
        configure(&mut self.textarea);
    }

    pub fn content(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        self.textarea.input(tui_textarea::Input::from(event.clone()))
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" MARKDOWN ")
            .border_style(if focused {
                theme::border_focused()
            } else {
                theme::border_unfocused()
            })
            .style(theme::base());

        if focused {
            self.textarea
                .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        } else {
            self.textarea.set_cursor_style(Style::default());
        }

        self.textarea.set_block(block);
        f.render_widget(&self.textarea, area);
    }
}

fn configure(ta: &mut TextArea<'static>) {
    ta.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    ta.set_line_number_style(Style::default().fg(Color::DarkGray));
    ta.set_style(Style::default().bg(Color::Black).fg(Color::White));
    let _ = ta.set_search_pattern(r"^#{1,6} .+");
    ta.set_search_style(theme::header_md());
}
