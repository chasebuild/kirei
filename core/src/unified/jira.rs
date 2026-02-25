use crate::unified::{
    ProviderClient, ProviderId, UnifiedCreateParams, UnifiedError, UnifiedIssue, UnifiedListQuery,
};

pub struct JiraClient;

impl JiraClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ProviderClient for JiraClient {
    fn provider(&self) -> ProviderId {
        ProviderId::Jira
    }

    async fn list(&self, _query: UnifiedListQuery) -> Result<Vec<UnifiedIssue>, UnifiedError> {
        Err(UnifiedError::NotImplemented(ProviderId::Jira))
    }

    async fn create(&self, _params: UnifiedCreateParams) -> Result<UnifiedIssue, UnifiedError> {
        Err(UnifiedError::NotImplemented(ProviderId::Jira))
    }
}
