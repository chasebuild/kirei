use crate::unified::{
    ProviderClient, ProviderId, UnifiedCreateParams, UnifiedError, UnifiedIssue, UnifiedListQuery,
};

pub struct TrelloClient;

impl TrelloClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ProviderClient for TrelloClient {
    fn provider(&self) -> ProviderId {
        ProviderId::Trello
    }

    async fn list(&self, _query: UnifiedListQuery) -> Result<Vec<UnifiedIssue>, UnifiedError> {
        Err(UnifiedError::NotImplemented(ProviderId::Trello))
    }

    async fn create(&self, _params: UnifiedCreateParams) -> Result<UnifiedIssue, UnifiedError> {
        Err(UnifiedError::NotImplemented(ProviderId::Trello))
    }
}
