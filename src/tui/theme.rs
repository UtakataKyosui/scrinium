use ratatui::style::{Color, Modifier, Style};

pub const BRAND_CYAN: Color = Color::Rgb(0, 191, 255);
pub const BRAND_BLUE: Color = Color::Rgb(0, 128, 255);
pub const BG: Color = Color::Black;
pub const FG: Color = Color::White;
pub const DIM_FG: Color = Color::DarkGray;

pub fn border_focused() -> Style {
    Style::default().fg(BRAND_CYAN).add_modifier(Modifier::BOLD)
}

pub fn border_unfocused() -> Style {
    Style::default().fg(BRAND_BLUE)
}

pub fn status_bar() -> Style {
    Style::default().bg(BRAND_BLUE).fg(FG).add_modifier(Modifier::BOLD)
}

pub fn header_md() -> Style {
    Style::default().fg(BRAND_CYAN).add_modifier(Modifier::BOLD)
}

pub fn label_active() -> Style {
    Style::default().fg(BRAND_CYAN).add_modifier(Modifier::BOLD)
}

pub fn label_inactive() -> Style {
    Style::default().fg(DIM_FG)
}

pub fn selected_file() -> Style {
    Style::default().fg(BRAND_CYAN).add_modifier(Modifier::BOLD)
}

pub fn error_style() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

pub fn base() -> Style {
    Style::default().bg(BG).fg(FG)
}
