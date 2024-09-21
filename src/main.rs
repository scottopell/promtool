use anyhow::Result;
use reqwest::blocking::Client;
use clap::Parser;
use std::io;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The Prometheus metrics endpoint URL
    #[arg(value_name = "ENDPOINT")]
    endpoint: String,
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

fn run_tui(content: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("Prometheus Metrics")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new(content).block(block);
            f.render_widget(paragraph, size);
        })?;

        if let Event::Key(key) = event::read()? {
            if let KeyCode::Char('q') = key.code {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let metrics = fetch_prometheus_metrics(&args.endpoint)?;
    run_tui(&metrics)?;
    Ok(())
}
