//! OpenAI-compatible API types
//!
//! These types mirror the OpenAI Chat Completions API for compatibility
//! with existing OpenAI client libraries.

use serde::{Deserialize, Serialize};
use chrono::Utc;

/// A chat message in OpenAI format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role: "system", "user", "assistant"
    pub role: String,

    /// Message content
    pub content: String,

    /// Optional name for the participant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Chat completion request (OpenAI-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model to use (mapped to agent)
    pub model: String,

    /// Messages in the conversation
    pub messages: Vec<ChatMessage>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Sampling temperature (0-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Number of completions to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Whether to stream responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// User identifier for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// A single choice in a chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// Index of this choice
    pub index: u32,

    /// The generated message
    pub message: ChatMessage,

    /// Reason for stopping: "stop", "length", "content_filter"
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,

    /// Tokens in the completion
    pub completion_tokens: u32,

    /// Total tokens used
    pub total_tokens: u32,
}

/// Chat completion response (OpenAI-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Unique ID for this completion
    pub id: String,

    /// Object type: "chat.completion"
    pub object: String,

    /// Unix timestamp of creation
    pub created: i64,

    /// Model used
    pub model: String,

    /// Generated completions
    pub choices: Vec<ChatCompletionChoice>,

    /// Token usage (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

impl ChatCompletionResponse {
    /// Create a new response with a single message
    pub fn new(id: String, model: String, content: String) -> Self {
        Self {
            id,
            object: "chat.completion".to_string(),
            created: Utc::now().timestamp(),
            model,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content,
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        }
    }
}

/// Streaming delta for a chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionDelta {
    /// Role (only in first delta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Content delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// A single choice in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamChoice {
    /// Index of this choice
    pub index: u32,

    /// The delta
    pub delta: ChatCompletionDelta,

    /// Reason for stopping
    pub finish_reason: Option<String>,
}

/// Streaming chat completion chunk (OpenAI-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Unique ID for this completion
    pub id: String,

    /// Object type: "chat.completion.chunk"
    pub object: String,

    /// Unix timestamp of creation
    pub created: i64,

    /// Model used
    pub model: String,

    /// Choices (deltas)
    pub choices: Vec<ChatCompletionStreamChoice>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model ID
    pub id: String,

    /// Object type: "model"
    pub object: String,

    /// Unix timestamp of creation
    pub created: i64,

    /// Owner/organization
    pub owned_by: String,
}

/// List of models response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    /// Object type: "list"
    pub object: String,

    /// Available models
    pub data: Vec<Model>,
}

/// Error response (OpenAI-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

impl ErrorResponse {
    pub fn new(message: String, error_type: &str) -> Self {
        Self {
            error: ErrorDetail {
                message,
                error_type: error_type.to_string(),
                param: None,
                code: None,
            },
        }
    }
}

/// Session list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionInfo>,
}

/// Session info for list responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub title: Option<String>,
    pub message_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

/// Create session request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub system_prompt: Option<String>,
    pub title: Option<String>,
}

/// Send message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// Send message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub role: String,
    pub content: String,
}
