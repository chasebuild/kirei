use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JiraError {
    #[error("missing credentials")]
    MissingCredentials,
    #[error("server URL is required")]
    ServerUrlRequired,
    #[error("project is required")]
    ProjectRequired,
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JiraConfig {
    pub server_url: Option<String>,
    pub default_project: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: String,
    pub url: Option<String>,
}

impl JiraIssue {
    pub fn from_json(value: &serde_json::Value) -> Self {
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let key = value
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let summary = value
            .get("fields")
            .and_then(|f| f.get("summary"))
            .and_then(|v| v.as_str())
            .unwrap_or("untitled")
            .to_string();

        let description = value
            .get("fields")
            .and_then(|f| f.get("description"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let status = value
            .get("fields")
            .and_then(|f| f.get("status"))
            .and_then(|s| s.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let url = value
            .get("self")
            .and_then(|v| v.as_str())
            .map(|s| s.replace("/rest/api/3/issue/", "/browse/"))
            .map(String::from);

        Self {
            id,
            key,
            summary,
            description,
            status,
            url,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct JiraProject {
    pub id: String,
    pub key: String,
    pub name: String,
}

pub struct JiraClient {
    http: Client,
    server_url: String,
    token: String,
    default_project: Option<String>,
}

impl JiraClient {
    pub fn new(token: String, server_url: String, default_project: Option<String>) -> Self {
        Self {
            http: Client::new(),
            server_url,
            token,
            default_project,
        }
    }

    pub fn with_project(token: String, server_url: String, project: String) -> Self {
        Self {
            http: Client::new(),
            server_url,
            token,
            default_project: Some(project),
        }
    }

    pub fn config(&self) -> Option<&String> {
        self.default_project.as_ref()
    }

    fn auth_header(&self) -> String {
        format!("Basic {}", base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", "email", self.token)))
    }

    fn resolve_project(&self, override_project: Option<&String>) -> Result<String, JiraError> {
        override_project
            .cloned()
            .or_else(|| self.default_project.clone())
            .ok_or(JiraError::ProjectRequired)
    }

    pub async fn list_issues(&self, project: Option<String>) -> Result<Vec<JiraIssue>, JiraError> {
        let project_key = self.resolve_project(project.as_ref())?;
        
        let jql = format!("project = {} AND status != Done ORDER BY created DESC", project_key);
        let url = format!("{}/rest/api/3/search?jql={}&maxResults=50", 
            self.server_url,
            urlencoding::encode(&jql)
        );

        let response = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        let issues = body
            .get("issues")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(JiraIssue::from_json).collect())
            .unwrap_or_default();

        Ok(issues)
    }

    pub async fn create_issue(&self, project: Option<String>, summary: &str, description: Option<&str>) -> Result<JiraIssue, JiraError> {
        let project_key = self.resolve_project(project.as_ref())?;
        
        let url = format!("{}/rest/api/3/issue", self.server_url);

        let mut fields = serde_json::json!({
            "project": {
                "key": project_key
            },
            "summary": summary,
            "issuetype": {
                "name": "Task"
            }
        });

        if let Some(desc) = description {
            fields["description"] = serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": desc
                    }]
                }]
            });
        }

        let payload = serde_json::json!({ "fields": fields });

        let response = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let issue: serde_json::Value = response.json().await?;
        Ok(JiraIssue::from_json(&issue))
    }

    pub async fn list_projects(&self) -> Result<Vec<JiraProject>, JiraError> {
        let url = format!("{}/rest/api/3/project", self.server_url);

        let response = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let projects: Vec<JiraProject> = response.json().await?;
        Ok(projects)
    }
}
