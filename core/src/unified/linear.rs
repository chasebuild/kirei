use crate::unified::{
    ProviderClient, ProviderId, UnifiedCreateParams, UnifiedError, UnifiedIssue, UnifiedListQuery,
};
use reqwest::Client;
use serde_json::{Value, json};

const LINEAR_GRAPHQL: &str = "https://api.linear.app/graphql";

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

    fn workspace_variable(&self, override_workspace: Option<&String>) -> Option<String> {
        override_workspace
            .cloned()
            .or_else(|| self.default_workspace.clone())
    }

    fn map_node(&self, value: &Value) -> UnifiedIssue {
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
        let state = value
            .get("state")
            .and_then(|state| state.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let url = value.get("url").and_then(Value::as_str).map(str::to_string);

        UnifiedIssue {
            id,
            title,
            state,
            url,
            provider: ProviderId::Linear,
            raw_payload: value.clone(),
        }
    }

    fn extract_nodes(response: &Value) -> Result<Vec<Value>, UnifiedError> {
        response
            .get("data")
            .and_then(|data| data.get("issues"))
            .and_then(|issues| issues.get("nodes"))
            .and_then(Value::as_array)
            .map(|nodes| nodes.clone())
            .ok_or_else(|| UnifiedError::UnexpectedResponse(response.to_string()))
    }
}

#[async_trait::async_trait]
impl ProviderClient for LinearClient {
    fn provider(&self) -> ProviderId {
        ProviderId::Linear
    }

    async fn list(&self, query: UnifiedListQuery) -> Result<Vec<UnifiedIssue>, UnifiedError> {
        let payload = json!({
            "query": r#"
                query($workspaceId: String) {
                    issues(first: 20, filter: { state: { type: { neq: "completed" } } }) {
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
                "workspaceId": self.workspace_variable(query.workspace.as_ref())
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
        let nodes = Self::extract_nodes(&body)?;
        Ok(nodes.into_iter().map(|node| self.map_node(&node)).collect())
    }

    async fn create(&self, params: UnifiedCreateParams) -> Result<UnifiedIssue, UnifiedError> {
        let workspace = self.workspace_variable(params.workspace.as_ref());
        let input = serde_json::json!({
            "title": params.title,
            "description": params.body,
            "teamId": workspace,
        });
        let payload = json!({
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
            .ok_or_else(|| UnifiedError::UnexpectedResponse(body.to_string()))?;

        Ok(self.map_node(issue))
    }
}
