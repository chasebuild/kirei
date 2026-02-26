use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrelloError {
    #[error("missing credentials")]
    MissingCredentials,
    #[error("API key is required")]
    ApiKeyRequired,
    #[error("board is required")]
    BoardRequired,
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TrelloConfig {
    pub default_board: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TrelloCard {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub list_name: String,
    pub url: Option<String>,
}

impl TrelloCard {
    pub fn from_json(value: &serde_json::Value, list_name: &str) -> Self {
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let name = value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("untitled")
            .to_string();

        let description = value
            .get("desc")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let url = value.get("url").and_then(|v| v.as_str()).map(String::from);

        Self {
            id,
            name,
            description,
            list_name: list_name.to_string(),
            url,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrelloBoard {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrelloList {
    pub id: String,
    pub name: String,
}

pub struct TrelloClient {
    http: Client,
    token: String,
    api_key: String,
    default_board: Option<String>,
}

impl TrelloClient {
    pub fn new(token: String, api_key: String, default_board: Option<String>) -> Self {
        Self {
            http: Client::new(),
            token,
            api_key,
            default_board,
        }
    }

    pub fn with_board(token: String, api_key: String, board: String) -> Self {
        Self {
            http: Client::new(),
            token,
            api_key,
            default_board: Some(board),
        }
    }

    pub fn config(&self) -> Option<&String> {
        self.default_board.as_ref()
    }

    fn auth_params(&self) -> Vec<(&str, &str)> {
        vec![
            ("key", self.api_key.as_str()),
            ("token", self.token.as_str()),
        ]
    }

    fn resolve_board(&self, override_board: Option<&String>) -> Result<String, TrelloError> {
        override_board
            .cloned()
            .or_else(|| self.default_board.clone())
            .ok_or(TrelloError::BoardRequired)
    }

    pub async fn list_cards(&self, board: Option<String>) -> Result<Vec<TrelloCard>, TrelloError> {
        let board_id = self.resolve_board(board.as_ref())?;
        
        let url = format!(
            "https://api.trello.com/1/boards/{}/cards",
            board_id
        );

        let response = self
            .http
            .get(&url)
            .query(&self.auth_params())
            .send()
            .await?;

        let cards: Vec<serde_json::Value> = response.json().await?;
        
        // Get lists for board to map card to list name
        let lists = self.list_lists(Some(board_id.clone())).await.unwrap_or_default();
        let list_map: std::collections::HashMap<String, String> = lists
            .into_iter()
            .map(|l| (l.id, l.name))
            .collect();

        Ok(cards.into_iter().map(|card| {
            let list_name = card.get("idList")
                .and_then(|v| v.as_str())
                .and_then(|id| list_map.get(id))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            TrelloCard::from_json(&card, &list_name)
        }).collect())
    }

    pub async fn create_card(&self, board: Option<String>, name: &str, description: Option<&str>) -> Result<TrelloCard, TrelloError> {
        let board_id = self.resolve_board(board.as_ref())?;
        
        // Get the first list on the board
        let lists = self.list_lists(Some(board_id.clone())).await?;
        let list_id = lists.first()
            .map(|l| l.id.clone())
            .ok_or_else(|| TrelloError::Configuration("No lists found on board".to_string()))?;

        let url = format!("https://api.trello.com/1/cards");

        let mut params = self.auth_params();
        params.push(("name", name));
        params.push(("idList", &list_id));
        if let Some(desc) = description {
            params.push(("desc", desc));
        }

        let response = self
            .http
            .post(&url)
            .query(&params)
            .send()
            .await?;

        let card: serde_json::Value = response.json().await?;
        Ok(TrelloCard::from_json(&card, ""))
    }

    pub async fn list_boards(&self) -> Result<Vec<TrelloBoard>, TrelloError> {
        let url = "https://api.trello.com/1/members/me/boards";

        let response = self
            .http
            .get(url)
            .query(&self.auth_params())
            .send()
            .await?;

        let boards: Vec<TrelloBoard> = response.json().await?;
        Ok(boards)
    }

    pub async fn list_lists(&self, board: Option<String>) -> Result<Vec<TrelloList>, TrelloError> {
        let board_id = self.resolve_board(board.as_ref())?;
        
        let url = format!(
            "https://api.trello.com/1/boards/{}/lists",
            board_id
        );

        let response = self
            .http
            .get(&url)
            .query(&self.auth_params())
            .send()
            .await?;

        let lists: Vec<TrelloList> = response.json().await?;
        Ok(lists)
    }
}
