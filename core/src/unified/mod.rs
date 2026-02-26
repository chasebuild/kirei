pub mod github;
pub mod jira;
pub mod linear;
pub mod trello;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    Github,
    Linear,
    Trello,
    Jira,
}

impl Default for ProviderId {
    fn default() -> Self {
        ProviderId::Github
    }
}

impl ProviderId {
    pub fn env_var(&self) -> &'static str {
        match self {
            ProviderId::Github => "KIREI_GITHUB_TOKEN",
            ProviderId::Linear => "KIREI_LINEAR_TOKEN",
            ProviderId::Trello => "KIREI_TRELLO_TOKEN",
            ProviderId::Jira => "KIREI_JIRA_TOKEN",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderId::Github => "GitHub",
            ProviderId::Linear => "Linear",
            ProviderId::Trello => "Trello",
            ProviderId::Jira => "Jira",
        }
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for ProviderId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(ProviderId::Github),
            "linear" => Ok(ProviderId::Linear),
            "trello" => Ok(ProviderId::Trello),
            "jira" => Ok(ProviderId::Jira),
            _ => Err(format!("unknown provider '{}'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedIssue {
    pub id: String,
    pub title: String,
    pub state: String,
    pub url: Option<String>,
    pub provider: ProviderId,
    pub raw_payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: ProviderId,
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub project_id: String,
    pub provider: ProviderId,
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedListQuery {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCreateParams {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProjectQuery {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCreateProjectParams {
    pub workspace: Option<String>,
    pub repo: Option<String>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTaskQuery {
    pub project_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCreateTaskParams {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMoveTaskParams {
    pub task_id: String,
    pub target_status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UnifiedError {
    #[error("missing credentials for {0}")]
    MissingToken(ProviderId),
    #[error("provider {0} is not implemented yet")]
    NotImplemented(ProviderId),
    #[error("provider response is malformed: {0}")]
    UnexpectedResponse(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("configuration error: {0}")]
    Configuration(String),
}

#[async_trait::async_trait]
pub trait ProviderClient: Send + Sync {
    fn provider(&self) -> ProviderId;

    async fn list(&self, query: UnifiedListQuery) -> Result<Vec<UnifiedIssue>, UnifiedError>;

    async fn create(&self, params: UnifiedCreateParams) -> Result<UnifiedIssue, UnifiedError>;

    // Project management
    async fn list_projects(
        &self,
        query: UnifiedProjectQuery,
    ) -> Result<Vec<UnifiedProject>, UnifiedError>;

    async fn create_project(
        &self,
        params: UnifiedCreateProjectParams,
    ) -> Result<UnifiedProject, UnifiedError>;

    // Task management
    async fn list_tasks(&self, query: UnifiedTaskQuery) -> Result<Vec<UnifiedTask>, UnifiedError>;

    async fn create_task(
        &self,
        params: UnifiedCreateTaskParams,
    ) -> Result<UnifiedTask, UnifiedError>;

    async fn move_task(
        &self,
        params: UnifiedMoveTaskParams,
    ) -> Result<UnifiedTask, UnifiedError>;
}

impl UnifiedIssue {
    pub fn display_summary(&self) -> String {
        format!(
            "{} [{}] {} ({})",
            self.provider.display_name(),
            self.state,
            self.title,
            self.raw_payload
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or("no-url")
        )
    }
}
