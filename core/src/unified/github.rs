use crate::unified::{
    ProviderClient, ProviderId, UnifiedCreateParams, UnifiedError, UnifiedIssue, UnifiedListQuery,
};
use reqwest::Client;
use serde_json::{Value, json};

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

    fn resolve_repo(
        &self,
        override_repo: Option<&String>,
    ) -> Result<(String, String), UnifiedError> {
        let repo = override_repo
            .cloned()
            .or_else(|| self.default_repo.clone())
            .ok_or_else(|| UnifiedError::Configuration("repository is required".into()))?;
        let mut segments = repo.splitn(2, '/');
        let owner = segments
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| UnifiedError::Configuration("repository owner missing".into()))?;
        let repo = segments
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| UnifiedError::Configuration("repository name missing".into()))?;
        Ok((owner.to_string(), repo.to_string()))
    }

    fn map_issue(&self, value: Value) -> UnifiedIssue {
        let number = value
            .get("number")
            .and_then(Value::as_i64)
            .map(|n| n.to_string())
            .unwrap_or_else(|| {
                value
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            });
        let title = value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("untitled")
            .to_string();
        let state = value
            .get("state")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let url = value
            .get("html_url")
            .and_then(Value::as_str)
            .map(str::to_string);

        UnifiedIssue {
            id: number,
            title,
            state,
            url,
            provider: ProviderId::Github,
            raw_payload: value,
        }
    }
}

#[async_trait::async_trait]
impl ProviderClient for GitHubClient {
    fn provider(&self) -> ProviderId {
        ProviderId::Github
    }

    async fn list(&self, query: UnifiedListQuery) -> Result<Vec<UnifiedIssue>, UnifiedError> {
        let (owner, repo) = self.resolve_repo(query.repo.as_ref())?;
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues?state=open&per_page=20",
            owner, repo
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
            .map(|issue| self.map_issue(issue))
            .collect())
    }

    async fn create(&self, params: UnifiedCreateParams) -> Result<UnifiedIssue, UnifiedError> {
        let (owner, repo) = self.resolve_repo(params.repo.as_ref())?;
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let body = json!({
            "title": params.title,
            "body": params.body,
        });

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .header("User-Agent", "kirei-cli")
            .json(&body)
            .send()
            .await?;

        let issue: Value = response.json().await?;
        Ok(self.map_issue(issue))
    }
}
