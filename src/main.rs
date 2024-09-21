use anyhow::Result;
use reqwest::blocking::Client;
use clap::Parser;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The Prometheus metrics endpoint URL
    #[arg(value_name = "ENDPOINT")]
    endpoint: String,
}

struct App {
    metrics: Vec<String>,
    scroll: u16,
}

fn fetch_prometheus_metrics(url: &str) -> Result<String> {
    let url = if !url.starts_with("http") {
        format!("http://{url}")
    } else {
        url.to_string()
    };

    let client = Client::new();
    let response = client.get(url).send()?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(response.error_for_status().unwrap_err().into());
    }
    Ok(response.text()?)
}


fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => app.scroll = app.scroll.saturating_add(1),
                KeyCode::Up => app.scroll = app.scroll.saturating_sub(1),
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(f.area());

    let metrics: Vec<ListItem> = app
        .metrics
        .iter()
        .map(|m| {
            ListItem::new(Line::from(vec![Span::styled(
                m.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )]))
        })
        .collect();

    let metrics_list = List::new(metrics)
        .block(Block::default().borders(Borders::ALL).title("Metrics"))
        .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
        .highlight_symbol(">> ");

    f.render_stateful_widget(metrics_list, chunks[1], &mut ratatui::widgets::ListState::default().with_selected(Some(app.scroll as usize)));
}

fn main() -> Result<()> {
    let args = Args::parse();
    let metrics = fetch_prometheus_metrics(&args.endpoint)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App {
        metrics: metrics.lines().map(String::from).collect(),
        scroll: 0,
    };

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
