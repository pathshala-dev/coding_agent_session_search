use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn search_bar(query: &str) -> Paragraph<'static> {
    Paragraph::new(Line::from(Span::raw(format!("/ {}", query))))
        .block(Block::default().title("Search").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Left)
}
