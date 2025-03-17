mod widgets;
mod clients;
mod config;

use std::time::Duration;

use clients::github;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{Event, EventStream, KeyCode, KeyEventKind}, layout::{Constraint, Layout}, style::Stylize, text::Line, DefaultTerminal, Frame
};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal).await;
    app_result
}

#[derive(Debug, Default)]
struct App {
    should_quit: bool,
    pull_requests: widgets::pull_requests::PullRequestListWidget,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let settings = config::Settings::new().expect("Could not parse config");

        let github_client = github::get_github_client(&settings.github.github_token)?;

        self.pull_requests.run(github_client, settings.github);

        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval  = tokio::time::interval(period);
        let mut events = EventStream::new();

        while !self.should_quit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event),
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [title_area, body_area] = vertical.areas(frame.area());
        let title = Line::from("Productui").centered().bold();
        frame.render_widget(title, title_area);
        frame.render_widget(&self.pull_requests, body_area);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Char('j') | KeyCode::Down => self.pull_requests.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => self.pull_requests.scroll_up(),
                    KeyCode::Enter => self.pull_requests.open_pr(),
                    // TODO System to change current widget
                    _ => {}
                }
            }
        }
    }
}