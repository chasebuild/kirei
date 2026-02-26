use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

const LINEAR_GRAPHQL: &str = "https://api.linear.app/graphql";

#[derive(Error, Debug)]
pub enum LinearError {
    #[error("missing credentials")]
    MissingCredentials,
    #[error("workspace is required")]
    WorkspaceRequired,
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LinearConfig {
    pub default_workspace: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LinearIssue {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub url: Option<String>,
}

impl LinearIssue {
    pub fn from_json(value: &Value) -> Self {
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        let title = value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("untitled")
            .to_string();

        let description = value
            .get("description")
            .and_then(Value::as_str)
            .map(String::from);

        let state = value
            .get("state")
            .and_then(|state| state.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let url = value.get("url").and_then(Value::as_str).map(String::from);

        Self {
            id,
            title,
            description,
            state,
            url,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LinearWorkspace {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LinearTeam {
    pub id: String,
    pub name: String,
    pub key: String,
}

pub struct LinearClient {
    http: Client,
    token: String,
    default_workspace: Option<String>,
}

impl LinearClient {
    pub fn new(token: String, default_workspace: Option<String>) -> Self {
        Self {
            http: Client::new(),
            token,
            default_workspace,
        }
    }

    pub fn with_workspace(token: String, workspace: String) -> Self {
        Self {
            http: Client::new(),
            token,
            default_workspace: Some(workspace),
        }
    }

    pub fn config(&self) -> Option<&String> {
        self.default_workspace.as_ref()
    }

    fn workspace_variable(&self, override_workspace: Option<&String>) -> Option<String> {
        override_workspace
            .cloned()
            .or_else(|| self.default_workspace.clone())
    }

    pub async fn list_issues(&self, workspace: Option<String>) -> Result<Vec<LinearIssue>, LinearError> {
        let workspace_id = self.workspace_variable(workspace.as_ref());
        
        let payload = serde_json::json!({
            "query": r#"
                query($workspaceId: String) {
                    issues(first: 50, filter: { state: { type: { neq: "completed" } } }) {
                        nodes {
                            id
                            title
                            description
                            url
                            state {
                                name
                            }
                        }
                    }
                }
            "#,
            "variables": {
                "workspaceId": workspace_id
            }
        });

        let response = self
            .http
            .post(LINEAR_GRAPHQL)
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        let body: Value = response.json().await?;
        let nodes = body
            .get("data")
            .and_then(|data| data.get("issues"))
            .and_then(|issues| issues.get("nodes"))
            .and_then(Value::as_array)
            .map(|nodes| nodes.clone())
            .ok_or_else(|| LinearError::Configuration("Failed to parse response".to_string()))?;

        Ok(nodes.iter().map(|node| LinearIssue::from_json(node)).collect())
    }

    pub async fn create_issue(&self, workspace: Option<String>, title: &str, body: Option<&str>) -> Result<LinearIssue, LinearError> {
        let workspace_id = self.workspace_variable(workspace.as_ref());
        
        let input = serde_json::json!({
            "title": title,
            "description": body,
            "teamId": workspace_id,
        });
        
        let payload = serde_json::json!({
            "query": r#"
                mutation IssueCreate($input: IssueCreateInput!) {
                    issueCreate(input: $input) {
                        success
                        issue {
                            id
                            title
                            description
                            url
                            state {
                                name
                            }
                        }
                    }
                }
            "#,
            "variables": {
                "input": input
            }
        });

        let response = self
            .http
            .post(LINEAR_GRAPHQL)
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        let body: Value = response.json().await?;
        let issue = body
            .get("data")
            .and_then(|data| data.get("issueCreate"))
            .and_then(|create| create.get("issue"))
            .ok_or_else(|| LinearError::Configuration("Failed to create issue".to_string()))?;

        Ok(LinearIssue::from_json(issue))
    }

    pub async fn list_workspaces(&self) -> Result<Vec<LinearWorkspace>, LinearError> {
        let payload = serde_json::json!({
            "query": r#"
                query {
                    organizations {
                        nodes {
                            id
                            name
                            slug
                        }
                    }
                }
            "#
        });

        let response = self
            .http
            .post(LINEAR_GRAPHQL)
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        let body: Value = response.json().await?;
        let nodes = body
            .get("data")
            .and_then(|data| data.get("organizations"))
            .and_then(|orgs| orgs.get("nodes"))
            .and_then(Value::as_array)
            .map(|nodes| nodes.clone())
            .ok_or_else(|| LinearError::Configuration("Failed to parse workspaces".to_string()))?;

        Ok(nodes.iter().filter_map(|node| {
            Some(LinearWorkspace {
                id: node.get("id")?.as_str()?.to_string(),
                name: node.get("name")?.as_str()?.to_string(),
                slug: node.get("slug")?.as_str()?.to_string(),
            })
        }).collect())
    }

    pub async fn list_teams(&self, workspace: Option<String>) -> Result<Vec<LinearTeam>, LinearError> {
        let workspace_id = self.workspace_variable(workspace.as_ref());
        
        let payload = serde_json::json!({
            "query": r#"
                query($workspaceId: String!) {
                    teams(first: 50, filter: { organization: { id: { eq: $workspaceId } } }) {
                        nodes {
                            id
                            name
                            key
                        }
                    }
                }
            "#,
            "variables": {
                "workspaceId": workspace_id
            }
        });

        let response = self
            .http
            .post(LINEAR_GRAPHQL)
            .bearer_auth(&self.token)
            .json(&payload)
            .send()
            .await?;

        let body: Value = response.json().await?;
        let nodes = body
            .get("data")
            .and_then(|data| data.get("teams"))
            .and_then(|teams| teams.get("nodes"))
            .and_then(Value::as_array)
            .map(|nodes| nodes.clone())
            .ok_or_else(|| LinearError::Configuration("Failed to parse teams".to_string()))?;

        Ok(nodes.iter().filter_map(|node| {
            Some(LinearTeam {
                id: node.get("id")?.as_str()?.to_string(),
                name: node.get("name")?.as_str()?.to_string(),
                key: node.get("key")?.as_str()?.to_string(),
            })
        }).collect())
    }
}
