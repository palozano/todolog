use std::io::{self, Stdout};

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::domain::{Task, TaskId, TaskStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderMode {
    FullScreen,
    Inline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskListCommand {
    Quit,
    Open(TaskId),
}

pub(crate) fn run_task_list<F>(
    mut tasks: Vec<Task>,
    mode: RenderMode,
    mut set_status: F,
) -> Result<TaskListCommand, String>
where
    F: FnMut(&TaskId, TaskStatus) -> Result<Vec<Task>, String>,
{
    let mut session =
        TerminalSession::start(mode).map_err(|err| format!("failed to start TUI: {err}"))?;
    let mut selected = 0;

    loop {
        session
            .terminal
            .draw(|frame| {
                draw_task_list(frame, &tasks, selected);
            })
            .map_err(|err| format!("failed to draw TUI: {err}"))?;

        if let Event::Key(key) =
            event::read().map_err(|err| format!("failed to read terminal input: {err}"))?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(TaskListCommand::Quit),
                KeyCode::Char('j') | KeyCode::Down if !tasks.is_empty() => {
                    selected = (selected + 1).min(tasks.len() - 1);
                }
                KeyCode::Char('k') | KeyCode::Up if !tasks.is_empty() => {
                    selected = selected.saturating_sub(1);
                }
                KeyCode::PageDown if !tasks.is_empty() => {
                    selected = (selected + 10).min(tasks.len() - 1);
                }
                KeyCode::PageUp if !tasks.is_empty() => {
                    selected = selected.saturating_sub(10);
                }
                KeyCode::Home if !tasks.is_empty() => selected = 0,
                KeyCode::End if !tasks.is_empty() => selected = tasks.len() - 1,
                KeyCode::Enter if !tasks.is_empty() => {
                    return Ok(TaskListCommand::Open(tasks[selected].id.clone()))
                }
                KeyCode::Char('d') if !tasks.is_empty() => {
                    tasks = set_status(&tasks[selected].id, TaskStatus::Done)?;
                    selected = selected.min(tasks.len().saturating_sub(1));
                }
                KeyCode::Char('o') if !tasks.is_empty() => {
                    tasks = set_status(&tasks[selected].id, TaskStatus::Open)?;
                    selected = selected.min(tasks.len().saturating_sub(1));
                }
                _ => {}
            }
        }
    }
}

fn draw_task_list(frame: &mut Frame<'_>, tasks: &[Task], selected: usize) {
    let area = frame.area();
    let [header_area, content_area, help_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(area);

    render_header(frame, header_area, tasks);

    if tasks.is_empty() {
        render_empty_state(frame, content_area);
        render_help(frame, help_area);
        return;
    }

    let selected = selected.min(tasks.len() - 1);
    let (list_area, detail_area) = split_content(content_area);

    render_tasks(frame, list_area, tasks, selected);
    render_detail(frame, detail_area, &tasks[selected]);
    render_help(frame, help_area);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, tasks: &[Task]) {
    let open = tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Open)
        .count();
    let done = tasks.len().saturating_sub(open);
    let summary = Line::from(vec![
        Span::styled("todolog", Style::default().fg(Color::Cyan).bold()),
        Span::raw("  |  "),
        Span::styled(
            format!("{open} open"),
            Style::default().fg(Color::Green).bold(),
        ),
        Span::raw("  |  "),
        Span::styled(format!("{done} done"), Style::default().fg(Color::DarkGray)),
    ]);

    let header = Paragraph::new(summary).alignment(Alignment::Center);
    frame.render_widget(header, area);
}

fn split_content(area: Rect) -> (Rect, Rect) {
    if area.width >= 100 {
        let [list_area, detail_area] =
            Layout::horizontal([Constraint::Percentage(68), Constraint::Percentage(32)])
                .areas(area);
        (list_area, detail_area)
    } else {
        let [list_area, detail_area] =
            Layout::vertical([Constraint::Min(8), Constraint::Length(8)]).areas(area);
        (list_area, detail_area)
    }
}

fn render_tasks(frame: &mut Frame<'_>, area: Rect, tasks: &[Task], selected: usize) {
    let items = tasks.iter().map(task_item);
    let mut state = ListState::default().with_selected(Some(selected));
    let list = List::new(items)
        .block(titled_block("open tasks"))
        .highlight_symbol("  ")
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(142, 111, 40))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_stateful_widget(list, area, &mut state);
}

fn task_item(task: &Task) -> ListItem<'_> {
    let status_style = task.status.style();
    let marker_style = marker_style(task.marker.as_str());

    ListItem::new(Text::from(vec![
        Line::from(vec![
            Span::styled(format!("{:<5}", task.marker), marker_style.bold()),
            Span::raw("  "),
            Span::styled(task.text.to_string(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::raw("       "),
            Span::styled(task.status.label(), status_style),
            Span::raw("  "),
            Span::styled(task.id.to_string(), Style::default().fg(Color::DarkGray)),
            Span::raw("  "),
            Span::styled(
                format!("{}:{}", task.file, task.line),
                Style::default().fg(Color::Blue),
            ),
        ]),
    ]))
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, task: &Task) {
    let detail = Text::from(vec![
        Line::from(vec![
            Span::styled("ID       ", label_style()),
            Span::styled(
                task.id.to_string(),
                Style::default().fg(Color::White).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Status   ", label_style()),
            Span::styled(task.status.label(), task.status.style().bold()),
        ]),
        Line::from(vec![
            Span::styled("Marker   ", label_style()),
            Span::styled(
                task.marker.to_string(),
                marker_style(task.marker.as_str()).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("Location ", label_style()),
            Span::styled(
                format!("{}:{}", task.file, task.line),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            task.text.to_string(),
            Style::default().fg(Color::White),
        )),
    ]);

    let detail = Paragraph::new(detail)
        .block(titled_block("selected task"))
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, area);
}

fn render_empty_state(frame: &mut Frame<'_>, area: Rect) {
    let empty = Text::from(vec![
        Line::from(Span::styled(
            "No open tasks",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::raw(""),
        Line::from(Span::styled(
            "Run todolog scan . to refresh the task file, or press q to close.",
            Style::default().fg(Color::DarkGray),
        )),
    ]);
    let paragraph = Paragraph::new(empty)
        .block(titled_block("open tasks"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let help = Text::from(vec![
        Line::from(vec![
            Span::styled("j/k", key_style()),
            Span::raw(" move  "),
            Span::styled("PgUp/PgDn", key_style()),
            Span::raw(" jump  "),
            Span::styled("Enter", key_style()),
            Span::raw(" open"),
        ]),
        Line::from(vec![
            Span::styled("d", key_style()),
            Span::raw(" done  "),
            Span::styled("o", key_style()),
            Span::raw(" reopen  "),
            Span::styled("q/Esc", key_style()),
            Span::raw(" quit"),
        ]),
    ]);

    let paragraph = Paragraph::new(help)
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn base_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
}

fn titled_block(title: &'static str) -> Block<'static> {
    base_block().title(title).title_alignment(Alignment::Center)
}

fn label_style() -> Style {
    Style::default().fg(Color::DarkGray).bold()
}

fn key_style() -> Style {
    Style::default().fg(Color::Yellow).bold()
}

fn marker_style(marker: &str) -> Style {
    match marker {
        "FIXME" => Style::default().fg(Color::Red),
        "HACK" => Style::default().fg(Color::Yellow),
        "XXX" => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::Cyan),
    }
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    mode: RenderMode,
}

impl TerminalSession {
    fn start(mode: RenderMode) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if mode == RenderMode::FullScreen {
            execute!(stdout, EnterAlternateScreen)?;
        }
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        Ok(Self { terminal, mode })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        if self.mode == RenderMode::FullScreen {
            let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        }
        let _ = self.terminal.show_cursor();
    }
}

trait TaskStatusLabel {
    fn label(self) -> &'static str;
    fn style(self) -> Style;
}

impl TaskStatusLabel for TaskStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Done => "done",
        }
    }

    fn style(self) -> Style {
        match self {
            Self::Open => Style::default().fg(Color::Green),
            Self::Done => Style::default().fg(Color::DarkGray),
        }
    }
}
