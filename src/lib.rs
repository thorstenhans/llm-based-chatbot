use anyhow::Result;
use bytes::Bytes;
use http_body_util::BodyExt;

use spin_sdk::http::{FullBody, Method, Request, Response, StatusCode};
use spin_sdk::http_service;
use spin_sdk::key_value::Store;

mod config;
mod models;
use models::*;

use crate::config::AppConfig;
type HttpResponse = Response<FullBody<Bytes>>;


#[http_service]
async fn handle_request(req: Request) -> Result<HttpResponse> {
    let path = req.uri().path().trim_end_matches('/');
    match (req.method(), path) {
        (&Method::POST, "" | "/chat") => handle_chat(req).await,
        (&Method::GET, "/history") => handle_get_history(req).await,
        (&Method::DELETE, "/history") => handle_delete_history(req).await,
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(FullBody::new(Bytes::from("Not Found")))?
        ),
    }
}

fn internal_server_error(message: &str) -> anyhow::Result<HttpResponse> {
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(FullBody::new(Bytes::from(message.to_owned())))?
    )
}

fn bad_request(message: &str) -> anyhow::Result<HttpResponse> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(FullBody::new(Bytes::from(message.to_owned())))?
    )
}

async fn handle_chat(req: Request) -> Result<HttpResponse> {
    let body_bytes = req.into_body().collect().await?.to_bytes();
    let Ok(input) = serde_json::from_slice::<ChatInput>(&body_bytes) else {
            return bad_request("Invalid Payload");
    };

    let Ok(config) = AppConfig::try_from_variables().await else {
        return internal_server_error("Failed to load configuration");
    };

    let store = Store::open_default().await?;

    let mut history: Vec<ChatMessage> = match store.get_json(&input.session_id).await? {
        Some(messages) => messages,
        None => vec![ChatMessage {
            role: "system".into(),
            content: "You are a helpful, concise assistant.".into(),
        }],
    };

    history.push(ChatMessage {
        role: "user".into(),
        content: input.message,
    });

    let chat_req = ChatCompletionRequest {
        model: &config.model_name,
        messages: &history,
    };

    let endpoint = format!("{}/chat/completions", config.openai_base_url.trim_end_matches('/'));
    let payload = serde_json::to_vec(&chat_req)?;

    let http_req = Request::builder()
        .method(Method::POST)
        .uri(&endpoint)
        .header("authorization", format!("Bearer {}", config.openai_api_key))
        .header("content-type", "application/json")
        .body(FullBody::new(Bytes::from(payload)))?;

    let http_res = spin_sdk::http::send(http_req).await?;
    let status = http_res.status();
    let res_bytes = http_res.into_body().collect().await?.to_bytes();

    if status != StatusCode::OK {
        return Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body(FullBody::new(res_bytes))
            .unwrap());
    }

    let chat_res: ChatCompletionResponse = serde_json::from_slice(&res_bytes)?;
    let choice = chat_res
        .choices
        .first()
        .ok_or_else(|| anyhow::anyhow!("No choices returned by LLM"))?;

    history.push(choice.message.clone());
    store.set_json(&input.session_id, &history).await?;

    let output = ChatOutput {
        session_id: input.session_id,
        reply: choice.message.content.clone(),
    };

    let out_bytes = serde_json::to_vec(&output)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(FullBody::new(Bytes::from(out_bytes)))
        .unwrap())
}

async fn handle_get_history(req: Request) -> Result<HttpResponse> {
    let session_id = extract_session_id(&req);
    let Some(session_id) = session_id else {
        return bad_request("Bad Session ID");
    };

    let store = Store::open_default().await?;
    let history: Vec<ChatMessage> = store
        .get_json(&session_id)
        .await?
        .unwrap_or_default();

    let out_bytes = serde_json::to_vec(&history)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(FullBody::new(Bytes::from(out_bytes)))?)
}

async fn handle_delete_history(req: Request) -> Result<HttpResponse> {
    let session_id = extract_session_id(&req);
    let Some(session_id) = session_id else {
        return bad_request("Bad Session ID");
    };

    let store = Store::open_default().await?;
    store.delete(&session_id).await?;

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(FullBody::new(Bytes::new()))?)
}

fn extract_session_id(req: &Request) -> Option<String> {
    let query = req.uri().query()?;
    for pair in query.split('&') {
        let mut parts = pair.split('=');
        if let (Some(key), Some(val)) = (parts.next(), parts.next()) {
            if key == "session_id" {
                return Some(val.to_string());
            }
        }
    }
    None
}
