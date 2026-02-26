use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use rand::Rng;
use reqwest::Client;
use tiny_http::{Response, Server};
use url::Url;

const GITHUB_OAUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

pub struct GitHubOAuth {
    http: Client,
    client_id: String,
    client_secret: String,
}

impl GitHubOAuth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            http: Client::new(),
            client_id,
            client_secret,
        }
    }

    pub fn get_authorization_url(&self, redirect_port: u16) -> String {
        let state = generate_random_state();
        let redirect_uri = format!("http://localhost:{}/callback", redirect_port);

        let mut url = Url::parse(GITHUB_OAUTH_URL).unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("scope", "repo")
            .append_pair("state", &state);

        format!("{}&state={}", url, state)
    }

    pub async fn exchange_code_for_token(&self, code: &str) -> Result<String> {
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
        ];

        let response = self
            .http
            .post(GITHUB_TOKEN_URL)
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No access token in response"))?;

        Ok(access_token.to_string())
    }
}

pub fn start_callback_server(
    port: u16,
) -> Result<(mpsc::Receiver<String>, mpsc::Receiver<()>)> {
    let server = Server::http(format!("127.0.0.1:{}", port))
        .map_err(|e| anyhow::anyhow!("Failed to start server: {}", e))?;
    let (code_tx, code_rx) = mpsc::channel();
    let (close_tx, close_rx) = mpsc::channel();

    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let url = request.url();
            if url.starts_with("/callback?") {
                if let Some(code) = extract_code_from_url(url) {
                    let _ = code_tx.send(code);
                }
            }
            let response = Response::from_string(
                r#"<html><body><h1>Authentication Complete</h1><p>You can close this window and return to the terminal.</p></body></html>"#,
            )
            .with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap(),
            );
            let _ = request.respond(response);
            let _ = close_tx.send(());
        }
    });

    Ok((code_rx, close_rx))
}

fn extract_code_from_url(url: &str) -> Option<String> {
    let parsed = Url::parse(&format!("http://localhost{}", url)).ok()?;
    let code = parsed.query_pairs().find(|(k, _)| k == "code");
    code.map(|(_, v)| v.to_string())
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

pub fn wait_for_callback(close_rx: mpsc::Receiver<()>, timeout_secs: u64) -> bool {
    match close_rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(_) => true,
        Err(_) => false,
    }
}
