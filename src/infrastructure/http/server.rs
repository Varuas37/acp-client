//! OpenAI-compatible HTTP server
//!
//! Exposes agents via standard OpenAI API endpoints.

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use chrono::Utc;
use uuid::Uuid;

use crate::application::AcpClient;
use crate::domain::{Agent, AgentConfig};
use crate::error::Error;
use super::types::*;

/// Application state for the HTTP server
pub struct AppState<A: Agent + 'static> {
    pub client: AcpClient<A>,
    pub config: AgentConfig,
}

impl<A: Agent + 'static> AppState<A> {
    pub fn new(agent: A, config: AgentConfig) -> Self {
        Self {
            client: AcpClient::new(agent, config.clone()),
            config,
        }
    }
}

/// Create the OpenAI-compatible router with a generic agent
pub fn create_router<A: Agent + Clone + 'static>(state: Arc<AppState<A>>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // OpenAI-compatible endpoints
        .route("/v1/chat/completions", post(chat_completions::<A>))
        .route("/v1/models", get(list_models))
        .route("/v1/models/:model_id", get(get_model))

        // Session management endpoints
        .route("/v1/sessions", get(list_sessions::<A>))
        .route("/v1/sessions", post(create_session::<A>))
        .route("/v1/sessions/:session_id", get(get_session::<A>))
        .route("/v1/sessions/:session_id", delete(delete_session::<A>))
        .route("/v1/sessions/:session_id/messages", post(send_message::<A>))

        // Health check
        .route("/health", get(health_check))

        .layer(cors)
        .with_state(state)
}

/// POST /v1/chat/completions - OpenAI-compatible chat completion
async fn chat_completions<A: Agent + Clone + 'static>(
    State(state): State<Arc<AppState<A>>>,
    Json(request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let model = request.model.clone();
    let model_for_response = model.clone();
    let messages = request.messages;

    // Build prompt from messages
    let prompt = messages
        .iter()
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Run ACP in blocking thread due to LocalSet requirements
    let agent = state.client.agent().clone();
    let config = state.config.clone();

    let result = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::spawn(e.to_string()))?;

        rt.block_on(async {
            let client = AcpClient::new(agent, config);
            client.send_prompt(&prompt).await
        })
    }).await;

    match result {
        Ok(Ok(content)) => {
            let response = ChatCompletionResponse::new(
                format!("chatcmpl-{}", Uuid::new_v4()),
                model_for_response,
                content,
            );
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(Err(e)) => {
            let error = ErrorResponse::new(e.to_string(), "api_error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Task failed: {}", e), "internal_error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// GET /v1/models - List available models
async fn list_models() -> impl IntoResponse {
    let models = vec![
        Model {
            id: "default".to_string(),
            object: "model".to_string(),
            created: Utc::now().timestamp(),
            owned_by: "acp-client".to_string(),
        },
    ];

    Json(ModelsResponse {
        object: "list".to_string(),
        data: models,
    })
}

/// GET /v1/models/:model_id - Get model info
async fn get_model(Path(model_id): Path<String>) -> impl IntoResponse {
    Json(Model {
        id: model_id,
        object: "model".to_string(),
        created: Utc::now().timestamp(),
        owned_by: "acp-client".to_string(),
    })
}

/// GET /v1/sessions - List all sessions
async fn list_sessions<A: Agent + 'static>(
    State(state): State<Arc<AppState<A>>>,
) -> impl IntoResponse {
    let sessions = state.client.sessions().list().await;
    let session_infos: Vec<SessionInfo> = sessions
        .iter()
        .map(|s| SessionInfo {
            id: s.id.clone(),
            title: s.title.clone(),
            message_count: s.messages.len(),
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        })
        .collect();

    Json(SessionListResponse {
        sessions: session_infos,
    })
}

/// POST /v1/sessions - Create a new session
async fn create_session<A: Agent + 'static>(
    State(state): State<Arc<AppState<A>>>,
    Json(request): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    let mut session = state.client.create_session(request.system_prompt).await;

    if let Some(title) = request.title {
        session.title = Some(title);
        let _ = state.client.sessions().update(session.clone()).await;
    }

    (StatusCode::CREATED, Json(session))
}

/// GET /v1/sessions/:session_id - Get session details
async fn get_session<A: Agent + 'static>(
    State(state): State<Arc<AppState<A>>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.client.sessions().get(&session_id).await {
        Ok(session) => (StatusCode::OK, Json(serde_json::to_value(session).unwrap())).into_response(),
        Err(_) => {
            let error = ErrorResponse::new(
                format!("Session not found: {}", session_id),
                "not_found",
            );
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

/// DELETE /v1/sessions/:session_id - Delete a session
async fn delete_session<A: Agent + 'static>(
    State(state): State<Arc<AppState<A>>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.client.sessions().delete(&session_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => {
            let error = ErrorResponse::new(
                format!("Session not found: {}", session_id),
                "not_found",
            );
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

/// POST /v1/sessions/:session_id/messages - Send a message in a session
async fn send_message<A: Agent + Clone + 'static>(
    State(state): State<Arc<AppState<A>>>,
    Path(session_id): Path<String>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let content = request.content.clone();
    let content_for_history = content.clone();

    // Check if session exists
    if state.client.sessions().get(&session_id).await.is_err() {
        let error = ErrorResponse::new(
            format!("Session not found: {}", session_id),
            "not_found",
        );
        return (StatusCode::NOT_FOUND, Json(error)).into_response();
    }

    let agent = state.client.agent().clone();
    let config = state.config.clone();

    // Run in blocking thread
    let result = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::spawn(e.to_string()))?;

        rt.block_on(async {
            let client = AcpClient::new(agent, config);
            client.send_prompt(&content).await
        })
    }).await;

    match result {
        Ok(Ok(response)) => {
            // Update session with messages
            let _ = state.client.sessions().add_message(
                &session_id,
                crate::domain::Message::user(content_for_history),
            ).await;
            let _ = state.client.sessions().add_message(
                &session_id,
                crate::domain::Message::assistant(response.clone()),
            ).await;

            (StatusCode::OK, Json(SendMessageResponse {
                role: "assistant".to_string(),
                content: response,
            })).into_response()
        }
        Ok(Err(e)) => {
            let error = ErrorResponse::new(e.to_string(), "api_error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse::new(format!("Task failed: {}", e), "internal_error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

/// GET /health - Health check
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "acp-client"
    }))
}

/// Start the server on the given port with a specific agent
pub async fn start_server<A: Agent + Clone + 'static>(
    agent: A,
    config: AgentConfig,
    port: u16,
) -> std::io::Result<()> {
    let state = Arc::new(AppState::new(agent, config));
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Server listening on port {}", port);

    axum::serve(listener, app).await
}
