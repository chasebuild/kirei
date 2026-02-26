use crate::unified::{ProviderClient, ProviderId, UnifiedCreateParams, UnifiedError, UnifiedIssue, UnifiedListQuery, UnifiedProject, UnifiedTask, UnifiedTaskStatus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct TrelloBoard {
    id: String,
    name: String,
    closed: bool,
    prefs: BoardPrefs,
}

#[derive(Debug, Serialize, Deserialize)]
struct BoardPrefs {
    permission_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrelloCard {
    id: String,
    name: String,
    desc: String,
    idList: String,
    idBoard: String,
    url: String,
    labels: Vec<TrelloLabel>,
    due: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrelloLabel {
    id: String,
    name: String,
    color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrelloList {
    id: String,
    name: String,
    idBoard: String,
    closed: bool,
}

pub struct TrelloClient {
    client: Client,
    base_url: String,
    token: String,
}

impl TrelloClient {
    pub fn new() -> Self {
        let base_url = "https://api.trello.com/1".to_string();
        let token = std::env::var("TRELLO_TOKEN").unwrap_or_else(|_| "".to_string());
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    async fn get_boards(&self) -> Result<Vec<TrelloBoard>, UnifiedError> {
        let url = format!("{}/members/me/boards?fields=id,name,closed,prefs&token={}", self.base_url, self.token);
        
        let res = self.client.get(url).send().await?;
        if res.status().is_success() {
            let boards: Vec<TrelloBoard> = res.json().await?;
            Ok(boards.into_iter().filter(|b| !b.closed).collect())
        } else {
            Err(UnifiedError::ProviderError("Failed to fetch boards".to_string()))
        }
    }

    async fn get_lists(&self, board_id: &str) -> Result<Vec<TrelloList>, UnifiedError> {
        let url = format!("{}/boards/{}/lists?fields=id,name,idBoard,closed&token={}", self.base_url, board_id, self.token);
        
        let res = self.client.get(url).send().await?;
        if res.status().is_success() {
            let lists: Vec<TrelloList> = res.json().await?;
            Ok(lists.into_iter().filter(|l| !l.closed).collect())
        } else {
            Err(UnifiedError::ProviderError("Failed to fetch lists".to_string()))
        }
    }

    async fn get_cards(&self, board_id: &str) -> Result<Vec<TrelloCard>, UnifiedError> {
        let url = format!("{}/boards/{}/cards?fields=id,name,desc,idList,idBoard,url,labels,due&token={}", self.base_url, board_id, self.token);
        
        let res = self.client.get(url).send().await?;
        if res.status().is_success() {
            let cards: Vec<TrelloCard> = res.json().await?;
            Ok(cards)
        } else {
            Err(UnifiedError::ProviderError("Failed to fetch cards".to_string()))
        }
    }

    async fn create_board(&self, name: &str) -> Result<TrelloBoard, UnifiedError> {
        let mut body = HashMap::new();
        body.insert("name", name);
        body.insert("defaultLists", "true");
        
        let url = format!("{}/boards?token={}", self.base_url, self.token);
        
        let res = self.client.post(url).json(&body).send().await?;
        if res.status().is_success() {
            let board: TrelloBoard = res.json().await?;
            Ok(board)
        } else {
            Err(UnifiedError::ProviderError("Failed to create board".to_string()))
        }
    }

    async fn create_card(&self, list_id: &str, name: &str, desc: &str) -> Result<TrelloCard, UnifiedError> {
        let mut body = HashMap::new();
        body.insert("idList", list_id);
        body.insert("name", name);
        body.insert("desc", desc);
        
        let url = format!("{}/cards?token={}", self.base_url, self.token);
        
        let res = self.client.post(url).json(&body).send().await?;
        if res.status().is_success() {
            let card: TrelloCard = res.json().await?;
            Ok(card)
        } else {
            Err(UnifiedError::ProviderError("Failed to create card".to_string()))
        }
    }

    async fn move_card(&self, card_id: &str, list_id: &str) -> Result<(), UnifiedError> {
        let mut body = HashMap::new();
        body.insert("idList", list_id);
        
        let url = format!("{}/cards/{}?token={}", self.base_url, card_id, self.token);
        
        let res = self.client.put(url).json(&body).send().await?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(UnifiedError::ProviderError("Failed to move card".to_string()))
        }
    }
}

#[async_trait::async_trait]
impl ProviderClient for TrelloClient {
    fn provider(&self) -> ProviderId {
        ProviderId::Trello
    }

    async fn list_projects(&self) -> Result<Vec<UnifiedProject>, UnifiedError> {
        let boards = self.get_boards().await?;
        let projects = boards.into_iter().map(|board| UnifiedProject {
            id: board.id,
            name: board.name,
            provider: ProviderId::Trello,
            url: Some(format!("https://trello.com/b/{}", board.id)),
            description: Some(format!("Trello board (permission level: {})", board.prefs.permission_level)),
            archived: board.closed,
        }).collect();
        
        Ok(projects)
    }

    async fn create_project(&self, name: &str) -> Result<UnifiedProject, UnifiedError> {
        let board = self.create_board(name).await?;
        Ok(UnifiedProject {
            id: board.id,
            name: board.name,
            provider: ProviderId::Trello,
            url: Some(format!("https://trello.com/b/{}", board.id)),
            description: Some(format!("Trello board (permission level: {})", board.prefs.permission_level)),
            archived: board.closed,
        })
    }

    async fn list_tasks(&self, project_id: &str) -> Result<Vec<UnifiedTask>, UnifiedError> {
        let cards = self.get_cards(project_id).await?;
        let tasks = cards.into_iter().map(|card| UnifiedTask {
            id: card.id,
            name: card.name,
            provider: ProviderId::Trello,
            project_id: Some(project_id.to_string()),
            url: Some(card.url.clone()),
            description: Some(card.desc.clone()),
            status: UnifiedTaskStatus::Active,
            due_date: card.due.clone(),
            labels: card.labels.into_iter().map(|label| label.name).collect(),
            archived: false,
        }).collect();
        
        Ok(tasks)
    }

    async fn create_task(&self, project_id: &str, name: &str, desc: &str) -> Result<UnifiedTask, UnifiedError> {
        let lists = self.get_lists(project_id).await?;
        let default_list = lists.first().ok_or_else(|| UnifiedError::ProviderError("No lists found in board".to_string()))?;
        
        let card = self.create_card(&default_list.id, name, desc).await?;
        Ok(UnifiedTask {
            id: card.id,
            name: card.name,
            provider: ProviderId::Trello,
            project_id: Some(project_id.to_string()),
            url: Some(card.url.clone()),
            description: Some(card.desc.clone()),
            status: UnifiedTaskStatus::Active,
            due_date: card.due.clone(),
            labels: card.labels.into_iter().map(|label| label.name).collect(),
            archived: false,
        })
    }

    async fn move_task(&self, task_id: &str, project_id: &str, target_status: &str) -> Result<(), UnifiedError> {
        let lists = self.get_lists(project_id).await?;
        let target_list = lists.into_iter().find(|list| list.name == target_status).ok_or_else(||
            UnifiedError::ProviderError(format!("List '{}' not found in board", target_status))
        )?;
        
        self.move_card(task_id, &target_list.id).await?;
        Ok(())
    }

    async fn list(&self, query: UnifiedListQuery) -> Result<Vec<UnifiedIssue>, UnifiedError> {
        let projects = self.list_projects().await?;
        let mut issues = Vec::new();
        
        for project in projects {
            let tasks = self.list_tasks(&project.id).await?;
            for task in tasks {
                issues.push(UnifiedIssue {
                    id: task.id,
                    name: task.name,
                    provider: ProviderId::Trello,
                    project_id: Some(project.id.clone()),
                    url: task.url.clone(),
                    description: task.description.clone(),
                    status: task.status.clone(),
                    due_date: task.due_date.clone(),
                    labels: task.labels.clone(),
                    archived: task.archived,
                });
            }
        }
        
        Ok(issues)
    }

    async fn create(&self, params: UnifiedCreateParams) -> Result<UnifiedIssue, UnifiedError> {
        let project_id = params.project_id.as_deref().ok_or_else(||
            UnifiedError::ProviderError("Project ID required for task creation".to_string())
        )?;
        
        let task = self.create_task(project_id, &params.name, &params.description.unwrap_or_default()).await?;
        Ok(UnifiedIssue {
            id: task.id,
            name: task.name,
            provider: ProviderId::Trello,
            project_id: Some(project_id.to_string()),
            url: task.url.clone(),
            description: task.description.clone(),
            status: task.status.clone(),
            due_date: task.due_date.clone(),
            labels: task.labels.clone(),
            archived: task.archived,
        })
    }
}