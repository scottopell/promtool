use anyhow::Result;
use openmetrics_parser::{PrometheusType, PrometheusValue};
use reqwest::blocking::Client;
use clap::Parser;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::Stylize, text::Text, widgets::{Row, Table}};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders},
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
    endpoint: String,
    latest_metrics: Result<openmetrics_parser::MetricsExposition<PrometheusType, PrometheusValue>, openmetrics_parser::ParseError>,
    scroll: u16,
}

fn fetch_prometheus_text(url: &str) -> Result<String> {
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

    match &app.latest_metrics {
        Ok(latest_metrics) => {
            let metrics: Vec<Row> = latest_metrics.families
                .iter()
                .map(|m| {
                    let (name, fam) = m;

                    // For each metricfamily, I want to check if all samples are from a single labelset
                    // ie, is there a single logical metric series within this metricfamily?
                    // or are there multiple?
                    // If there is a single, that means I can display a labelset and sample value on the same line
                    // if there are multiple, I'd want to open either a side pane or a tree, not sure.
                    // So for now, if there are multiple, I guess lets just display '(multiple labelsets)'


                    let m_str = fam.metrics_as_string().unwrap_or(String::from("Couldn't render metrics"));

                    Row::new(vec![
                        Text::from(name.clone()).bold().alignment(Alignment::Left),
                        Text::from(format!("{}", fam.family_type)).alignment(Alignment::Center),
                        Text::from(m_str).alignment(Alignment::Right),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ];

            let metrics_list = Table::new(metrics, widths)
                .block(Block::default().borders(Borders::ALL).title("Metrics"))
                .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
                .highlight_symbol(">> ");

            f.render_stateful_widget(metrics_list, chunks[1], &mut ratatui::widgets::TableState::default().with_selected(Some(app.scroll as usize)));
        },
        Err(e) => {
            let widget = Span::styled(format!("Metrics from {} could not be parsed: {}", app.endpoint, e), Style::default().add_modifier(Modifier::SLOW_BLINK));
            f.render_widget(widget, chunks[1]);
        }
    }

}

fn main() -> Result<()> {
    let args = Args::parse();
    let metric_text = fetch_prometheus_text(&args.endpoint)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let latest_metrics = openmetrics_parser::prometheus::parse_prometheus(&metric_text);
    
    let app = App {
        endpoint: args.endpoint,
        latest_metrics,
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
