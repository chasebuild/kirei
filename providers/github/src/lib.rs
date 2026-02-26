use anyhow::Result;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

pub use crate::oauth::{GitHubOAuth, start_callback_server, wait_for_callback};

pub mod oauth;

#[derive(Error, Debug)]
pub enum GitHubError {
    #[error("missing credentials")]
    MissingCredentials,
    #[error("repository is required")]
    RepositoryRequired,
    #[error("repository owner missing")]
    OwnerMissing,
    #[error("repository name missing")]
    RepoNameMissing,
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub default_repo: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Clone, Debug)]
pub struct GitHubRepository {
    pub owner: String,
    pub name: String,
}

impl GitHubRepository {
    pub fn new(owner: String, name: String) -> Self {
        Self { owner, name }
    }

    pub fn from_string(repo: &str) -> Result<Self, GitHubError> {
        let mut segments = repo.splitn(2, '/');
        let owner = segments
            .next()
            .filter(|s| !s.is_empty())
            .ok_or(GitHubError::OwnerMissing)?
            .to_string();
        let name = segments
            .next()
            .filter(|s| !s.is_empty())
            .ok_or(GitHubError::RepoNameMissing)?
            .to_string();
        Ok(Self { owner, name })
    }

    pub fn as_str(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct GitHubIssue {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: Option<String>,
}

impl GitHubIssue {
    pub fn from_json(value: Value) -> Self {
        let number = value
            .get("number")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        
        let id = value
            .get("id")
            .and_then(Value::as_i64)
            .map(|id| id.to_string())
            .unwrap_or_else(|| number.to_string());

        let title = value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("untitled")
            .to_string();

        let body = value
            .get("body")
            .and_then(Value::as_str)
            .map(String::from);

        let state = value
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let html_url = value
            .get("html_url")
            .and_then(Value::as_str)
            .map(String::from);

        Self {
            id,
            number,
            title,
            body,
            state,
            html_url,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitHubRepositoryInfo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub private: bool,
    pub owner: GitHubOwner,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitHubOwner {
    pub login: String,
    pub avatar_url: Option<String>,
}

pub struct GitHubClient {
    http: Client,
    token: String,
    default_repo: Option<String>,
}

impl GitHubClient {
    pub fn new(token: String, default_repo: Option<String>) -> Self {
        Self {
            http: Client::new(),
            token,
            default_repo,
        }
    }

    pub fn with_repo(token: String, repo: GitHubRepository) -> Self {
        Self {
            http: Client::new(),
            token,
            default_repo: Some(repo.as_str()),
        }
    }

    pub fn config(&self) -> Option<&String> {
        self.default_repo.as_ref()
    }

    fn resolve_repo(&self, override_repo: Option<&String>) -> Result<GitHubRepository, GitHubError> {
        let repo = override_repo
            .cloned()
            .or_else(|| self.default_repo.clone())
            .ok_or(GitHubError::RepositoryRequired)?;
        GitHubRepository::from_string(&repo)
    }

    pub async fn list_issues(&self, repo: Option<String>, state: Option<&str>) -> Result<Vec<GitHubIssue>, GitHubError> {
        let repo = self.resolve_repo(repo.as_ref())?;
        let state = state.unwrap_or("open");
        
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues?state={}&per_page=50",
            repo.owner, repo.name, state
        );

        let response = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .header("User-Agent", "kirei-cli")
            .send()
            .await?;

        let issues: Vec<Value> = response.json().await?;
        Ok(issues
            .into_iter()
            .filter(|issue| issue.get("pull_request").is_none()) // Filter out PRs
            .map(GitHubIssue::from_json)
            .collect())
    }

    pub async fn create_issue(&self, repo: Option<String>, title: &str, body: Option<&str>) -> Result<GitHubIssue, GitHubError> {
        let repo = self.resolve_repo(repo.as_ref())?;
        let url = format!("https://api.github.com/repos/{}/{}/issues", repo.owner, repo.name);
        
        let mut body_obj = serde_json::Map::new();
        body_obj.insert("title".to_string(), serde_json::Value::String(title.to_string()));
        if let Some(body) = body {
            body_obj.insert("body".to_string(), serde_json::Value::String(body.to_string()));
        }

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .header("User-Agent", "kirei-cli")
            .json(&body_obj)
            .send()
            .await?;

        let issue: Value = response.json().await?;
        Ok(GitHubIssue::from_json(issue))
    }

    pub async fn list_repositories(&self) -> Result<Vec<GitHubRepositoryInfo>, GitHubError> {
        let url = "https://api.github.com/user/repos?per_page=100&sort=updated";

        let response = self
            .http
            .get(url)
            .bearer_auth(&self.token)
            .header("User-Agent", "kirei-cli")
            .send()
            .await?;

        let repos: Vec<GitHubRepositoryInfo> = response.json().await?;
        Ok(repos)
    }

    pub async fn get_token_info(&self) -> Result<GitHubUser, GitHubError> {
        let url = "https://api.github.com/user";

        let response = self
            .http
            .get(url)
            .bearer_auth(&self.token)
            .header("User-Agent", "kirei-cli")
            .send()
            .await?;

        let user: GitHubUser = response.json().await?;
        Ok(user)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubOAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl Default for GitHubOAuthConfig {
    fn default() -> Self {
        Self {
            client_id: None,
            client_secret: None,
        }
    }
}

pub fn get_authorization_url(client_id: &str, redirect_port: u16) -> String {
    let state = generate_random_state();
    let redirect_uri = format!("http://localhost:{}/callback", redirect_port);

    let mut url = Url::parse("https://github.com/login/oauth/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", "repo")
        .append_pair("state", &state);

    format!("{}&state={}", url, state)
}

pub async fn exchange_code_for_token(client_id: &str, client_secret: &str, code: &str) -> Result<String, GitHubError> {
    let http = Client::new();
    
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
    ];

    let response = http
        .post("https://github.com/login/oauth/access_token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let access_token = json["access_token"]
        .as_str()
        .ok_or(GitHubError::MissingCredentials)?;

    Ok(access_token.to_string())
}

fn generate_random_state() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}
