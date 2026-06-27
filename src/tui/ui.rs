use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::{
    app::{App, AppMode},
    theme,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let full_area = f.area();
    app.last_full_area = full_area;

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(full_area);

    let main_area = outer[0];
    let status_area = outer[1];
    app.last_status_area = status_area;

    let h_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(main_area);

    let browser_area = h_split[0];
    let right_area = h_split[1];
    app.last_right_area = right_area;

    let v_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(right_area);

    let fm_area = v_split[0];
    let md_area = v_split[1];

    // Black background fill
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        full_area,
    );

    let focused_browser = matches!(app.mode, AppMode::Browse);
    let focused_fm = matches!(app.mode, AppMode::EditFrontmatter);
    let focused_md = matches!(app.mode, AppMode::EditMarkdown);

    app.browser.render(f, browser_area, focused_browser);
    app.fm_editor.render(f, fm_area, focused_fm);
    app.md_editor.render(f, md_area, focused_md);

    render_status_bar(f, status_area, app);

    // Trigger startup sweep effect only on the first rendered frame
    app.effects.trigger_startup(full_area);

    if app.mode == AppMode::HelpPopup {
        render_help_popup(f, full_area);
    }
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let mode_label = match &app.mode {
        AppMode::Browse => "BROWSE",
        AppMode::EditFrontmatter => "FRONTMATTER",
        AppMode::EditMarkdown => "MARKDOWN",
        AppMode::HelpPopup => "HELP",
    };

    let (row, col) = app.md_editor.cursor();
    let file_name = app
        .current_file
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("no file");
    let dirty = if app.is_dirty { "[*]" } else { "   " };

    let msg = if !app.status_message.is_empty() {
        format!("  {}", app.status_message)
    } else {
        String::new()
    };

    let hint = format!(" Ln {}, Col {} | ^S Save  F1 Help  ^Q Quit ", row + 1, col + 1);
    let left = format!(" {} | {}{}{}", mode_label, file_name, dirty, msg);

    let used = left.chars().count() + hint.chars().count();
    let pad = (area.width as usize).saturating_sub(used);

    let msg_style = if app.status_is_error {
        theme::error_style()
    } else {
        theme::status_bar()
    };

    let line = Line::from(vec![
        Span::styled(left, theme::status_bar()),
        Span::styled(" ".repeat(pad), theme::status_bar()),
        Span::styled(hint, msg_style),
    ]);

    f.render_widget(Paragraph::new(line), area);
}

fn render_help_popup(f: &mut Frame, area: Rect) {
    let popup_w: u16 = 54;
    let popup_h: u16 = 15;
    let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(x, y, popup_w, popup_h);

    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(""),
        Line::from("  Tab / Shift+Tab    Cycle panel focus"),
        Line::from("  j / k  or  ↑ ↓    Navigate file list"),
        Line::from("  Enter              Open selected file"),
        Line::from("  Ctrl+S             Save file"),
        Line::from("  Ctrl+Q / Esc       Quit"),
        Line::from("  F1                 Toggle this help"),
        Line::from(""),
        Line::from("  In Frontmatter:"),
        Line::from("    ↑ ↓ / Enter      Move between fields"),
        Line::from("  In Markdown:"),
        Line::from("    Standard text editing keys"),
        Line::from(""),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" HELP (F1 to close) ")
        .border_style(theme::border_focused())
        .style(theme::base());

    f.render_widget(Paragraph::new(help_text).block(block), popup_area);
}
