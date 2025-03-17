use std::sync::{ Arc, RwLock };

use octocrab::{ params::{ pulls::Sort, Direction }, Octocrab, Page };
use ratatui::{
    buffer::Buffer,
    layout::{ Constraint, Rect },
    style::{ Style, Stylize },
    text::Line,
    widgets::{ Block, HighlightSpacing, Row, StatefulWidget, Table, TableState, Widget },
};
use serde::Deserialize;
use crate::config::PRFilter;

use crate::config::GitHubSettings;

#[derive(Debug, Clone, Default)]
pub struct PullRequestListWidget {
    github_client: Octocrab,
    config: GitHubSettings,
    state: Arc<RwLock<PullRequestListState>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Repo(pub String, pub String);

#[derive(Debug, Default)]
pub struct PullRequestListState {
    pull_requests: Vec<PullRequest>,
    loading_state: LoadingState,
    authed_login: Option<String>,
    table_state: TableState,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PullRequest {
    title: String,
    repo: String,
    id: String,
    url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum LoadingState {
    #[default]
    Idle,
    Loading,
    Loaded,
    Error(String),
}

type OctoPullRequest = octocrab::models::pulls::PullRequest;

impl From<&OctoPullRequest> for PullRequest {
    fn from(pr: &OctoPullRequest) -> Self {
        Self {
            id: pr.number.to_string(),
            title: pr.title.as_ref().unwrap().to_string(),
            repo: pr.base.repo.as_ref().unwrap().name.clone(),
            url: pr.html_url.as_ref().map(ToString::to_string).unwrap_or_default(),
        }
    }
}

impl From<&PullRequest> for Row<'_> {
    fn from(pr: &PullRequest) -> Self {
        let pr = pr.clone();
        Row::new(vec![pr.title, pr.repo])
    }
}

impl PullRequestListWidget {
    pub fn run(&mut self, github_client: octocrab::Octocrab, config: GitHubSettings) {
        let mut this = self.clone();

        this.github_client = github_client;
        this.config = config;

        tokio::spawn(this.fetch_pulls());
    }

    async fn fetch_pulls(self) {
        self.set_loading_state(LoadingState::Loading);

        let authed_user = match self.github_client.current().user().await {
            Ok(a) => Some(a.login.clone()),
            Err(_) => None,
        };
        self.set_authed_user(authed_user);

        for r in self.config.repos.iter() {
            match
                self.github_client
                    .pulls(r.0.clone(), r.1.clone())
                    .list()
                    .state(octocrab::params::State::Open)
                    .sort(Sort::Updated)
                    .direction(Direction::Descending)
                    .send().await
            {
                Ok(page) => self.on_load(&page),
                Err(err) => self.on_err(&err),
            }
        }
    }

    // TODO Clean up/Refactor
    fn pr_filter(&self, pr: &OctoPullRequest, authed_user: &Option<String>) -> bool {
        let mut filters = self.config.filters.iter().map(|f| {
            match f {
                PRFilter::ReviewRequested =>
                    authed_user.is_none() ||
                        pr.requested_reviewers
                            .iter()
                            .flatten()
                            .any(|r| r.login == authed_user.clone().unwrap_or(String::new())),
                PRFilter::Mentions =>
                    authed_user.is_none() ||
                        pr.body
                            .as_ref()
                            .unwrap_or(&"".to_string())
                            .contains(
                                &format!("@{}", authed_user.clone().unwrap_or(String::new()))
                            ),
                PRFilter::Labels =>
                    self.config.labels.len() == 0 ||
                        pr.labels
                            .iter()
                            .flatten()
                            .any(|l| { self.config.labels.iter().any(|m| l.name == m.clone()) }),
                PRFilter::Assigned =>
                    authed_user.is_none() ||
                        pr.assignees
                            .iter()
                            .flatten()
                            .any(|a| a.login == authed_user.clone().unwrap_or(String::new())),
                PRFilter::Created =>
                    authed_user.is_none() ||
                        pr.user.as_ref().unwrap().login ==
                            authed_user.clone().unwrap_or(String::new()),
            }
        });

        filters.any(|f| f)
    }

    fn on_load(&self, page: &Page<OctoPullRequest>) {
        let mut state = self.state.write().unwrap();
        let authed_user = state.authed_login.clone();

        let prs = page.items
            .iter()
            .filter(|&pr| self.pr_filter(pr, &authed_user))
            .map(Into::into);

        state.loading_state = LoadingState::Loaded;

        state.pull_requests.extend(prs);
        state.pull_requests.sort();

        if !state.pull_requests.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn on_err(&self, err: &octocrab::Error) {
        self.set_loading_state(LoadingState::Error(err.to_string()));
    }

    fn set_loading_state(&self, state: LoadingState) {
        self.state.write().unwrap().loading_state = state;
    }

    fn set_authed_user(&self, authed_user: Option<String>) {
        self.state.write().unwrap().authed_login = authed_user;
    }

    pub fn scroll_down(&self) {
        self.state.write().unwrap().table_state.scroll_down_by(1);
    }

    pub fn scroll_up(&self) {
        self.state.write().unwrap().table_state.scroll_up_by(1);
    }

    pub fn open_pr(&self) {
        let state = self.state.read().unwrap();

        let selected_index = state.table_state.selected().unwrap();
        let selected = &state.pull_requests[selected_index];

        open::that(selected.url.clone()).unwrap();
    }

    // TODO add refresh function
}

impl Widget for &PullRequestListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        // a block with a right aligned title with the loading state on the right
        let loading_state = Line::from(format!("{:?}", state.loading_state)).right_aligned();

        let user = Line::from(state.authed_login.clone().unwrap_or(String::from("No Auth Token")));

        let block = Block::bordered()
            .title("Pull Requests")
            .title(user)
            .title(loading_state)
            .title_bottom("j/k to scroll, q to quit");

        // a table with the list of pull requests
        let rows = state.pull_requests.iter();
        let widths = [Constraint::Fill(1), Constraint::Max(49)];
        let table = Table::new(rows, widths)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(">>")
            .header(Row::new(vec!["Name", "Repo"]))
            .row_highlight_style(Style::new().on_blue());

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}
