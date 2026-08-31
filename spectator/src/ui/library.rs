use crate::app::{LibraryState, LIBRARY_PAGE_SIZE};
use crate::replay::ReplayEntry;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

struct ColumnWidths {
    name: usize,
    result: usize,
    points: usize,
    steps: usize,
    date: usize,
}

pub fn render(f: &mut Frame, library: &mut LibraryState, area: Rect) {
    f.render_widget(Clear, area);
    library.row_rects.clear();

    let block = Block::default()
        .title(Span::styled(
            " Replay Library ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(LIBRARY_PAGE_SIZE as u16 + 1),
            Constraint::Length(2),
        ])
        .split(inner);

    let page = library.page() + 1;
    let total_pages = library.total_pages();
    let header = format!(
        "  {} replay(s) — {}   page {page}/{total_pages}",
        library.entries.len(),
        library.replays_dir.display()
    );
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            header,
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[0],
    );

    let cols = column_widths(chunks[2].width);
    f.render_widget(
        Paragraph::new(column_header_line(&cols)).style(Style::default().fg(Color::Cyan)),
        chunks[1],
    );

    let mut lines: Vec<Line> = Vec::new();
    if library.entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No replays found. Record one with record_episode.py",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let start = library.page_start();
        let end = library.page_end();
        for (row_i, (i, entry)) in library
            .entries
            .iter()
            .enumerate()
            .skip(start)
            .take(end - start)
            .enumerate()
        {
            let selected = i == library.selected;
            let row_y = chunks[2].y + row_i as u16;
            library.row_rects.push(Rect {
                x: chunks[2].x,
                y: row_y,
                width: chunks[2].width,
                height: 1,
            });
            lines.push(format_row(entry, selected, library.confirm_delete, &cols));
        }
    }

    f.render_widget(Paragraph::new(lines), chunks[2]);

    let footer = if library.rename_active {
        "  Enter=save name  Esc=cancel"
    } else if library.confirm_delete {
        "  Enter=confirm delete  Esc=cancel"
    } else {
        "  Enter=watch  j/k=move  [/]=page  R=rename  r=refresh  d=delete  q=quit"
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer,
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[3],
    );

    if library.rename_active {
        render_rename_dialog(f, library, area);
    }
}

fn column_widths(area_width: u16) -> ColumnWidths {
    // Keep metadata columns tight to the name; don't stretch name across the terminal.
    let result = 6;
    let points = 6;
    let steps = 5;
    let date = 10;
    let gaps = 4;
    let prefix_marker = 4;
    let max_name = 28usize;
    let fixed = prefix_marker + result + points + steps + date + gaps;
    let available = area_width.saturating_sub(fixed as u16) as usize;
    let name = available.min(max_name).max(14);
    ColumnWidths {
        name,
        result,
        points,
        steps,
        date,
    }
}

fn column_header_line(cols: &ColumnWidths) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{:<width$}", "name", width = cols.name),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:^width$}", "result", width = cols.result),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", "points", width = cols.points),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", "steps", width = cols.steps),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", "date", width = cols.date),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ])
}

fn format_row(
    entry: &ReplayEntry,
    selected: bool,
    confirm_delete: bool,
    cols: &ColumnWidths,
) -> Line<'static> {
    let marker = if selected { "▶" } else { " " };
    let result = if entry.won { "WIN" } else { "LOSS" };
    let result_color = if entry.won { Color::Green } else { Color::Red };
    let date = if entry.recorded_at.is_empty() {
        "-".to_string()
    } else {
        entry.recorded_at.chars().take(10).collect()
    };

    let prefix = if confirm_delete && selected {
        Span::styled("! ", Style::default().fg(Color::Red))
    } else {
        Span::raw("  ")
    };
    let name_style = if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if entry.has_custom_name() {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    Line::from(vec![
        prefix,
        Span::raw(format!("{marker} ")),
        Span::styled(
            format!("{:<width$}", truncate(&entry.display_name(), cols.name), width = cols.name),
            name_style,
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:^width$}", result, width = cols.result),
            Style::default().fg(result_color),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", entry.score, width = cols.points),
            Style::default().fg(Color::White),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", entry.steps, width = cols.steps),
            Style::default().fg(Color::White),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:>width$}", date, width = cols.date),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

fn render_rename_dialog(f: &mut Frame, library: &LibraryState, area: Rect) {
    let w = 52u16.min(area.width.saturating_sub(4)).max(30);
    let h = 10u16;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);
    f.render_widget(Clear, rect);

    let input_width = w.saturating_sub(6) as usize;
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Replay name",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(vec![
            Span::raw("  ┌"),
            Span::raw("─".repeat(input_width + 2)),
            Span::raw("┐"),
        ]),
        Line::from(vec![
            Span::raw("  │ "),
            Span::styled(
                format!("{:<width$}", library.rename_input, width = input_width),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("│"),
        ]),
        Line::from(vec![
            Span::raw("  └"),
            Span::raw("─".repeat(input_width + 2)),
            Span::raw("┘"),
        ]),
    ];
    if let Some(err) = &library.rename_error {
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(Color::Red),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  letters, numbers, spaces, - _ .",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = Block::default()
        .title(Span::styled(
            " Rename Replay ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(Paragraph::new(lines).block(block), rect);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{end}…")
    }
}
