use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::HttpClient;
use crate::error::SignalHouseError;
use crate::request_options::RequestOptions;
use crate::response::SignalHouseResponse;

/// Data used to create a new agent profile.
///
/// Field names are snake_case in Rust but serialize to the camelCase keys the
/// `/agent/profiles` API expects. `group_id` and `name` are required; every
/// other field is optional and omitted from the request body when `None`, so
/// the API applies its own defaults (`status` -> `"active"`, etc.).
///
/// Model, voice and sampling settings are deliberately absent: they are per-channel
/// and live on [`UpsertAgentChannelSettingData`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentProfileData {
    /// The parent group id (at least 8 characters, starts with `G`). Required.
    pub group_id: String,
    /// Optional subgroup id (at least 8 characters, starts with `S`);
    /// `None`/omitted = group-level agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subgroup_id: Option<String>,
    /// The agent's name. Required.
    pub name: String,
    /// Lifecycle status: `"active"` or `"inactive"` (defaults to `"active"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Base system prompt composed into every conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Opening line the agent leads with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greeting: Option<String>,
    /// Freeform guardrail instructions appended to the composed prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardrails: Option<String>,
    /// Default send authority: `"review"` or `"autopilot"` (defaults to `"review"`).
    /// Deployments may override it per subgroup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_authority: Option<String>,
}

/// Data used to update an existing agent profile. Every field is optional; only
/// the fields present are changed. The agent's scope (`groupId`/`subgroupId`) is
/// immutable and therefore intentionally absent from this struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentProfileData {
    /// The agent's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Lifecycle status: `"active"` or `"inactive"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Base system prompt composed into every conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Opening line the agent leads with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greeting: Option<String>,
    /// Freeform guardrail instructions appended to the composed prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardrails: Option<String>,
    /// Default send authority: `"review"` or `"autopilot"`. Deployments may override it
    /// per subgroup. The read-only `publishStatus` and `publishedAt` fields change only
    /// through [`AgentsDomain::publish_agent_profile`] and
    /// [`AgentsDomain::unpublish_agent_profile`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_authority: Option<String>,
}

/// Data used to send a message to an agent (the app-native conversation endpoint).
///
/// Field names are snake_case in Rust but serialize to the camelCase keys the
/// `/agent/messages` API expects. `agent_profile_id`, `channel`, and `message` are
/// required; `conversation_id` and `contact_identifier` are optional and omitted from
/// the request body when `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendAgentMessageRequest {
    /// The agent to talk to. Required.
    pub agent_profile_id: String,
    /// The channel: `"webchat"`, `"sms"`, or `"voice"`. Required.
    pub channel: String,
    /// The user's message. Required.
    pub message: String,
    /// Continue this conversation; `None` starts a new one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    /// The far-end identifier (webchat visitor id, sender number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_identifier: Option<String>,
}

/// Body for starting a conversation without sending a message.
///
/// Field names are snake_case in Rust but serialize to the camelCase keys the
/// `/agent/conversations` API expects. `agent_profile_id` and `channel` are required;
/// every other field is omitted from the request body when `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartConversationRequest {
    /// The agent this conversation belongs to. Required.
    pub agent_profile_id: String,
    /// The channel: `"webchat"`, `"sms"`, or `"voice"`. Required.
    pub channel: String,
    /// The far-end identifier (visitor id, caller number).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_identifier: Option<String>,
    /// Telephony call id, for the voice channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// Arbitrary metadata stored with the conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Body for appending one message to a conversation.
///
/// `role` is explicit because this records a turn rather than generating one — an assistant
/// turn your own runtime produced is the normal case. `content` is optional so a tool-only
/// turn needs no prose.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendConversationMessageRequest {
    /// `"user"`, `"assistant"`, `"system"`, or `"tool"`. Required.
    pub role: String,
    /// What was said.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Tools invoked on this turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<serde_json::Value>>,
    /// Their results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<serde_json::Value>>,
    /// Time to first byte/audio, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttfb_ms: Option<i64>,
    /// Total turn latency, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<i64>,
    /// Whether the far end talked over this turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barge_in_occurred: Option<bool>,
}

/// Body for ending a conversation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndConversationRequest {
    /// `"completed"`, `"escalated"`, or `"abandoned"`. Defaults to `"completed"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Metadata merged onto the conversation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Data used to create or update (upsert) an agent's per-channel setting. Field names are
/// snake_case in Rust but serialize to the camelCase keys the API expects; every field is
/// optional and omitted from the request body when `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAgentChannelSettingData {
    /// Tools enabled on this channel (built-in keys or agentTool ids).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    /// Whether the agent answers on this channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Channel-specific prompt text appended to the agent's system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_prompt: Option<String>,
    /// Channel-specific greeting override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greeting: Option<String>,
    /// LLM backend for this channel: `"bedrock"`, `"openai"`, `"anthropic"`, or
    /// `"groq"` (defaults to `"bedrock"`). Model settings are per-channel because
    /// each channel is served by a different runtime and the model catalogues do
    /// not overlap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_provider: Option<String>,
    /// Concrete model id for the provider; `None` = provider default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,
    /// Sampling temperature (0–2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Voice identifier for spoken channels; ignored off the voice channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_id: Option<String>,
    /// Speaking rate multiplier (0.7–1.2); `None` = platform default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// Voice consistency (0–1); lower is more expressive and more variable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f64>,
    /// How closely synthesis tracks the source voice (0–1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_boost: Option<f64>,
    /// Speech synthesis model. Separate from `llm_model`, which decides what the agent
    /// says rather than how it is voiced; the two are chosen independently.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_model: Option<String>,
    /// How readily the agent decides the caller has finished: `"patient"`, `"normal"`,
    /// or `"eager"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_eagerness: Option<String>,
    /// Silence before the agent takes its turn: 1–300 seconds, or `-1` for no timeout.
    /// Values between 0 and 1 are rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_timeout_seconds: Option<f64>,
    /// Silence before the agent speaks first: 1–300 seconds, or `-1` for no timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_wait_seconds: Option<f64>,
    /// Silence before the call is ended outright (10–7200 seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_end_call_seconds: Option<f64>,
    /// Hard ceiling on one conversation (60–7200 seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_call_duration_seconds: Option<f64>,
    /// Whether a caller may talk over the greeting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_greeting_interruption: Option<bool>,
    /// Words the transcriber is biased toward — names, jargon, unusual spellings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyterms: Option<Vec<String>>,
    /// Ambient audio bed played under the call; `None` = silence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_sound: Option<AgentBackgroundSound>,
    /// Conversation workflow graph, in the voice runtime's own shape: `nodes` and `edges` keyed
    /// maps, plus optional `subgraphs` and `prevent_subagent_loops`. Node bodies are defined by
    /// the runtime and passed through as-is, so this is an untyped [`serde_json::Value`]. Voice
    /// channel only — sending one on webchat or sms is rejected.
    ///
    /// `None` leaves the stored graph alone; `Some(serde_json::Value::Null)` clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<serde_json::Value>,
}

/// Data used to deploy a group-level agent to a subgroup, or change that deployment.
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects;
/// a `None` field is omitted from the request body and leaves the stored value unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAgentDeploymentData {
    /// Per-subgroup override of the profile's default send authority. `None` omits the
    /// field; `Some(Some("review" | "autopilot"))` sets the override; `Some(None)` sends
    /// `null`, clearing the override so the deployment inherits the profile default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_authority_override: Option<Option<String>>,
    /// Whether this deployment is live (defaults to `true` on create).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Data used to assign an agent to answer a number (an endpoint binding).
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAgentEndpointData {
    /// The agent that answers the number. Required. The agent must be active, in the
    /// number's group, and either belong to the number's subgroup or be a group-level agent
    /// with an enabled deployment there.
    pub agent_profile_id: String,
    /// Which inbound messages the agent answers: `"all_inbound"` (the only mode, and the
    /// default). `None` omits the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer_mode: Option<String>,
}

/// Data used to pause an agent on one conversation (its number + one contact).
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects.
/// The default value serializes to `{}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseAgentConversationData {
    /// Why the conversation is paused: `"human_reply"` (a person replied by hand) or
    /// `"manual"` (the default). `None` omits the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Data used to approve a held agent reply (a reply draft).
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAgentReplyDraftData {
    /// Replacement text when a person edited the reply (1 to 10000 characters). `None`
    /// omits the field and sends the agent's draft as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Data used to claim a held agent reply for delivery through your own channel.
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimAgentReplyDraftData {
    /// Replacement text when a person edited the reply (1 to 10000 characters). `None`
    /// omits the field and claims the agent's draft as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Data used to complete (mark sent) a claimed agent reply.
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteAgentReplyDraftData {
    /// The claim being completed, as returned by `claim_agent_reply_draft`. Required.
    pub claim_id: String,
    /// Your channel's id for the sent message (up to 256 characters), stored as
    /// `sentMessageId`. `None` omits the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_message_id: Option<String>,
}

/// Data used to release a claimed agent reply that certainly did not go out.
/// Field names are snake_case in Rust but serialize to the camelCase keys the API expects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseAgentReplyDraftData {
    /// The claim being released, as returned by `claim_agent_reply_draft`. Required.
    pub claim_id: String,
    /// `Some(true)` closes the draft as failed instead of returning it to pending. `None`
    /// omits the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permanent: Option<bool>,
}

/// An ambient audio bed played under a call. Presets come from a fixed set provided by
/// the speech runtime; customer-uploaded audio is not supported. Serializes to camelCase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBackgroundSound {
    /// The ambient bed to play, e.g. `"office1"` or `"restaurant"`.
    pub preset: String,
    /// Playback level (0.01–1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
}

/// One day's open/close window in a tenant's business hours. Times are `"HH:MM"` (24h);
/// `closed` short-circuits open/close for that day. Serializes to camelCase keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessHoursEntry {
    /// Weekday: `"monday"`..`"sunday"`.
    pub day: String,
    /// When `Some(true)`, the day is fully closed (open/close ignored).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// Local opening time `"HH:MM"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<String>,
    /// Local closing time `"HH:MM"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close: Option<String>,
}

/// Data used to create or update (upsert) a scope's tenant AI settings. Field names are
/// snake_case in Rust but serialize to the camelCase keys the API expects; every field is
/// optional and omitted from the request body when `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTenantAiSettingsData {
    /// Brand/business name the agent represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    /// Freeform tone descriptor (e.g. `"professional"`, `"friendly"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    /// IANA timezone the business hours are evaluated in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Per-day open/close windows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_hours: Option<Vec<BusinessHoursEntry>>,
    /// Message used outside business hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_hours_message: Option<String>,
}

/// Data used to create a knowledge item. Field names are snake_case in Rust but serialize to
/// the camelCase keys the API expects. `group_id` and `text` are required; the raw text is
/// stored and Atlas Vector Search owns the embedding. Optional fields are omitted when `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKnowledgeItemData {
    /// The owning group (starts with `G`). Required.
    pub group_id: String,
    /// Subgroup scope (starts with `S`); `None` = group-level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subgroup_id: Option<String>,
    /// Restrict to one agent; `None` = shared across the tenant's agents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_profile_id: Option<String>,
    /// Short human label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The raw content Atlas auto-embeds and retrieves against. Required.
    pub text: String,
    /// How the item was added: `"manual"`, `"url"`, `"document"`, or `"import"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Whether the item is included in retrieval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Free-form per-item metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Data used to update a knowledge item. Every field is optional; the item's scope
/// (group/subgroup/agent) is immutable and intentionally absent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKnowledgeItemData {
    /// Short human label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The raw content Atlas auto-embeds and retrieves against.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// How the item was added: `"manual"`, `"url"`, `"document"`, or `"import"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Whether the item is included in retrieval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Free-form per-item metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Data used to create an agent tool. `group_id`, `name`, and `description` are required;
/// type-specific fields are required by the server for their `type` (webhook → `webhook_url`,
/// builtin → `builtin_key`, mcp → `mcp_server_url`). A webhook `webhook_url` must be a public
/// https URL. Field names are snake_case but serialize to the camelCase keys the API expects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentToolData {
    /// The owning group (starts with `G`). Required.
    pub group_id: String,
    /// Subgroup scope (starts with `S`); `None` = group-level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subgroup_id: Option<String>,
    /// Restrict to one agent; `None` = shared across the tenant's agents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_profile_id: Option<String>,
    /// Tool name exposed to the LLM. Required.
    pub name: String,
    /// What the tool does; the LLM reads this to decide when to call it. Required.
    pub description: String,
    /// `"builtin"`, `"webhook"`, or `"mcp"` (defaults to `"builtin"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The built-in this configures (for type `"builtin"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtin_key: Option<String>,
    /// JSON Schema for the tool's parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_schema: Option<serde_json::Value>,
    /// Per-tool configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    /// Customer endpoint the call is relayed to (for type `"webhook"`); must be public https.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    /// Auth config for the webhook call; stored, never echoed back. MUST serialize to
    /// `{ "headers"?: { name: value }, "signingSecret"?: string }` — any other shape (e.g.
    /// `{ "header", "value" }`) is rejected with a 400. `headers` are sent verbatim on every call
    /// (reserved/framing and `x-signalhouse-*` names are rejected); `signingSecret` makes us send
    /// `X-SignalHouse-Signature: sha256=HMAC-SHA256("{X-SignalHouse-Timestamp}.{rawBody}")` so the
    /// endpoint can verify the call and reject replays.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_auth: Option<serde_json::Value>,
    /// MCP server URL (for type `"mcp"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_url: Option<String>,
    /// Channels this tool is offered on (subset of the agent channels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    /// Whether the tool is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// Data used to update an agent tool. Every field is optional; the tool's scope
/// (group/subgroup/agent) is immutable and intentionally absent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentToolData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtin_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    /// `{ "headers"?: { name: value }, "signingSecret"?: string }`; other shapes are rejected (400).
    /// See [`CreateAgentToolData::webhook_auth`] for the header constraints and signing scheme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_auth: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// An agent profile as returned by the API — who the agent is, independent of how
/// it runs. Model, voice and sampling settings live on the per-channel setting, not
/// here. Deserialize a response's `data` into this with `serde_json::from_value`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentProfile {
    /// The group that owns this agent.
    pub group_id: String,
    /// The subgroup this agent belongs to; `None` = group-level.
    pub subgroup_id: Option<String>,
    /// Unique prefixed identifier (assigned by the counter service).
    pub agent_profile_id: String,
    /// Human-readable agent name.
    pub name: String,
    /// `"active"` or `"inactive"`.
    pub status: String,
    /// Base system prompt composed into every conversation.
    pub system_prompt: String,
    /// Opening line the agent leads with.
    pub greeting: String,
    /// Freeform guardrail instructions appended to the composed prompt.
    pub guardrails: String,
    /// Creation timestamp (ISO-8601).
    pub created_at: Option<String>,
    /// Last-update timestamp (ISO-8601).
    pub updated_at: Option<String>,
}

/// AI Agents domain — agent profile CRUD (customer-facing agent config mounted
/// under `/agent`). Accessed via `client.agents`.
pub struct AgentsDomain {
    http: Arc<HttpClient>,
}

impl AgentsDomain {
    pub(crate) fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    /// List the voices an agent can be configured to speak with.
    ///
    /// The catalog is platform-wide, not per-account, so this takes no scope. The
    /// `voiceId` of an entry is the value to set on a spoken channel's setting.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/voices`
    pub async fn get_agent_voices(
        &self,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        self.http
            .request_json("/agent/voices", Method::GET, None, options)
            .await
    }

    /// List the agent profiles under a group.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/profiles?...`
    /// `subgroup_id` narrows to the agents that subgroup can use — its own, plus the
    /// group-level agents shared with every subgroup.
    pub async fn get_agent_profiles(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/agent/profiles{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a single agent profile by id.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/profiles/:agentProfileId`
    pub async fn get_agent_profile(
        &self,
        agent_profile_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentProfileId", Some(agent_profile_id))])?;
        let safe = urlencoding::encode(agent_profile_id);
        self.http
            .request_json(
                &format!("/agent/profiles/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a new agent profile. `profile_data` must include `group_id` and
    /// `name`; every other field is optional (see [`CreateAgentProfileData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **POST** `/agent/profiles`
    pub async fn create_agent_profile(
        &self,
        profile_data: &CreateAgentProfileData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = serde_json::to_value(profile_data)?;
        self.http
            .request_json("/agent/profiles", Method::POST, Some(&body), options)
            .await
    }

    /// Update an existing agent profile. The scope (`group_id`/`subgroup_id`) is
    /// immutable; see [`UpdateAgentProfileData`] for the updatable fields.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/profiles/:id`
    pub async fn update_agent_profile(
        &self,
        id: &str,
        update_data: &UpdateAgentProfileData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = serde_json::to_value(update_data)?;
        self.http
            .request_json(
                &format!("/agent/profiles/{}", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Permanently delete an agent profile by its id.
    ///
    /// Permanently deletes the agent, its channel settings, and any tools and knowledge scoped to it, plus their provider-side counterparts. Tools and knowledge shared across the account are untouched. Conversation records survive — a transcript outlives the agent that produced it.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/profiles/:id`
    pub async fn delete_agent_profile(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/agent/profiles/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// Send a message to an agent and get its reply — the app-native conversation
    /// endpoint (webchat / SMS). Set `conversation_id` to continue an existing
    /// conversation, or leave it `None` to start a new one (see
    /// [`SendAgentMessageRequest`]). The agent's LLM tool loop runs server-side; the
    /// response carries `conversationId`, `conversationSessionId`, `reply`,
    /// `toolInvocations`, and `ok`.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/messages`
    pub async fn send_agent_message(
        &self,
        request: &SendAgentMessageRequest,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(request.agent_profile_id.as_str())),
            ("channel", Some(request.channel.as_str())),
            ("message", Some(request.message.as_str())),
        ])?;
        let body = serde_json::to_value(request)?;
        self.http
            .request_json("/agent/messages", Method::POST, Some(&body), options)
            .await
    }

    /// Start a conversation without sending a message.
    ///
    /// The counterpart to [`send_agent_message`](Self::send_agent_message) for callers that
    /// run the model themselves — a voice runtime, or your own LLM. Those record what was
    /// said; `send_agent_message` decides it. Both write the same conversation records.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/conversations`
    pub async fn start_conversation(
        &self,
        request: &StartConversationRequest,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(request.agent_profile_id.as_str())),
            ("channel", Some(request.channel.as_str())),
        ])?;
        let body = serde_json::to_value(request)?;
        self.http
            .request_json("/agent/conversations", Method::POST, Some(&body), options)
            .await
    }

    /// Read one conversation with a page of its messages, in order.
    ///
    /// Paged rather than whole: a transcript has no natural ceiling. `None` for `limit`
    /// returns 100 messages, not all of them.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/conversations/:conversationId`
    pub async fn get_conversation(
        &self,
        conversation_id: &str,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("conversationId", Some(conversation_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        let path = format!(
            "/agent/conversations/{}{}",
            urlencoding::encode(conversation_id),
            qs
        );
        self.http
            .request_json(&path, Method::GET, None, options)
            .await
    }

    /// Append one message to a conversation.
    ///
    /// Records a turn rather than generating one, so `role` is explicit — an assistant turn
    /// your own runtime produced is the normal case here.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/conversations/:conversationId/messages`
    pub async fn append_conversation_message(
        &self,
        conversation_id: &str,
        request: &AppendConversationMessageRequest,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("conversationId", Some(conversation_id)),
            ("role", Some(request.role.as_str())),
        ])?;
        let path = format!(
            "/agent/conversations/{}/messages",
            urlencoding::encode(conversation_id)
        );
        let body = serde_json::to_value(request)?;
        self.http
            .request_json(&path, Method::POST, Some(&body), options)
            .await
    }

    /// End a conversation, closing its open session.
    ///
    /// `status` records HOW it ended and cannot be recovered afterwards, so pass the one
    /// that actually happened. Ending an already-ended conversation is a no-op, so a retry
    /// after a dropped connection cannot overwrite it.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **PUT** `/agent/conversations/:conversationId`
    pub async fn end_conversation(
        &self,
        conversation_id: &str,
        request: &EndConversationRequest,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("conversationId", Some(conversation_id))])?;
        let path = format!(
            "/agent/conversations/{}",
            urlencoding::encode(conversation_id)
        );
        let body = serde_json::to_value(request)?;
        self.http
            .request_json(&path, Method::PUT, Some(&body), options)
            .await
    }

    /// List an agent's per-channel settings.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/profiles/:agentProfileId/channels`
    pub async fn get_agent_channel_settings(
        &self,
        agent_profile_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentProfileId", Some(agent_profile_id))])?;
        let safe = urlencoding::encode(agent_profile_id);
        self.http
            .request_json(
                &format!("/agent/profiles/{}/channels", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a single per-channel setting for an agent.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/profiles/:agentProfileId/channels/:channel`
    pub async fn get_agent_channel_setting(
        &self,
        agent_profile_id: &str,
        channel: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(agent_profile_id)),
            ("channel", Some(channel)),
        ])?;
        let safe_id = urlencoding::encode(agent_profile_id);
        let safe_channel = urlencoding::encode(channel);
        self.http
            .request_json(
                &format!("/agent/profiles/{}/channels/{}", safe_id, safe_channel),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create or update (upsert) an agent's per-channel setting. One row per (agent,
    /// channel); the channel is taken from the path (see [`UpsertAgentChannelSettingData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/profiles/:agentProfileId/channels/:channel`
    pub async fn upsert_agent_channel_setting(
        &self,
        agent_profile_id: &str,
        channel: &str,
        setting_data: &UpsertAgentChannelSettingData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(agent_profile_id)),
            ("channel", Some(channel)),
        ])?;
        let safe_id = urlencoding::encode(agent_profile_id);
        let safe_channel = urlencoding::encode(channel);
        let body = serde_json::to_value(setting_data)?;
        self.http
            .request_json(
                &format!("/agent/profiles/{}/channels/{}", safe_id, safe_channel),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Delete an agent's per-channel setting.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/profiles/:agentProfileId/channels/:channel`
    pub async fn delete_agent_channel_setting(
        &self,
        agent_profile_id: &str,
        channel: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(agent_profile_id)),
            ("channel", Some(channel)),
        ])?;
        let safe_id = urlencoding::encode(agent_profile_id);
        let safe_channel = urlencoding::encode(channel);
        self.http
            .request_json(
                &format!("/agent/profiles/{}/channels/{}", safe_id, safe_channel),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// List a group-level agent's deployments to subgroups. Each deployment carries
    /// `agentDeploymentId`, `groupId`, `subgroupId`, `agentProfileId`, `sendAuthorityOverride`,
    /// `sendAuthorityChangedBy`, `sendAuthorityChangedAt`, `enabled`, `createdAt`, and `updatedAt`.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/profiles/:agentProfileId/deployments`
    pub async fn get_agent_deployments(
        &self,
        agent_profile_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentProfileId", Some(agent_profile_id))])?;
        let safe = urlencoding::encode(agent_profile_id);
        self.http
            .request_json(
                &format!("/agent/profiles/{}/deployments", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Deploy a group-level agent to a subgroup, or change that deployment (upsert). One row
    /// per (agent, subgroup); the subgroup is taken from the path (see
    /// [`UpsertAgentDeploymentData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/profiles/:agentProfileId/deployments/:subgroupId`
    pub async fn upsert_agent_deployment(
        &self,
        agent_profile_id: &str,
        subgroup_id: &str,
        deployment_data: &UpsertAgentDeploymentData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(agent_profile_id)),
            ("subgroupId", Some(subgroup_id)),
        ])?;
        let safe_id = urlencoding::encode(agent_profile_id);
        let safe_subgroup_id = urlencoding::encode(subgroup_id);
        let body = serde_json::to_value(deployment_data)?;
        self.http
            .request_json(
                &format!(
                    "/agent/profiles/{}/deployments/{}",
                    safe_id, safe_subgroup_id
                ),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Remove an agent's deployment from a subgroup. Returns the removed deployment.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/profiles/:agentProfileId/deployments/:subgroupId`
    pub async fn delete_agent_deployment(
        &self,
        agent_profile_id: &str,
        subgroup_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentProfileId", Some(agent_profile_id)),
            ("subgroupId", Some(subgroup_id)),
        ])?;
        let safe_id = urlencoding::encode(agent_profile_id);
        let safe_subgroup_id = urlencoding::encode(subgroup_id);
        self.http
            .request_json(
                &format!(
                    "/agent/profiles/{}/deployments/{}",
                    safe_id, safe_subgroup_id
                ),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// List the endpoint bindings (which agent answers which number) in a group, ordered by
    /// number. Each binding carries `agentEndpointId`, `groupId`, `subgroupId`, `channel`,
    /// `endpointType`, `endpointValue`, `agentProfileId`, `answerMode`, `overrides`, `enabled`,
    /// `createdAt`, and `updatedAt`.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/endpoints?...`
    /// `subgroup_id`, `agent_profile_id` and `channel` (`"sms"`) narrow the list.
    pub async fn get_agent_endpoints(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        agent_profile_id: Option<&str>,
        channel: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
            ("agentProfileId", agent_profile_id.map(|s| s.to_string())),
            ("channel", channel.map(|s| s.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/agent/endpoints{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Assign an agent to answer a number (upsert keyed by channel + number). Assigning again
    /// replaces the previous agent, so a number has one agent per channel. Only SMS numbers can be bound:
    /// `channel` must be `"sms"`, and `endpoint_value` is the phone number as 10 to 15
    /// digits, country code included, no `+` (see [`UpsertAgentEndpointData`]). Returns the
    /// created or updated binding.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/endpoints/:channel/:endpointValue`
    pub async fn upsert_agent_endpoint(
        &self,
        channel: &str,
        endpoint_value: &str,
        endpoint_data: &UpsertAgentEndpointData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("channel", Some(channel)),
            ("endpointValue", Some(endpoint_value)),
            (
                "agentProfileId",
                Some(endpoint_data.agent_profile_id.as_str()),
            ),
        ])?;
        let safe_channel = urlencoding::encode(channel);
        let safe_endpoint_value = urlencoding::encode(endpoint_value);
        let body = serde_json::to_value(endpoint_data)?;
        self.http
            .request_json(
                &format!("/agent/endpoints/{}/{}", safe_channel, safe_endpoint_value),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Remove the agent assigned to a number. Returns the removed binding.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/endpoints/:channel/:endpointValue`
    pub async fn delete_agent_endpoint(
        &self,
        channel: &str,
        endpoint_value: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("channel", Some(channel)),
            ("endpointValue", Some(endpoint_value)),
        ])?;
        let safe_channel = urlencoding::encode(channel);
        let safe_endpoint_value = urlencoding::encode(endpoint_value);
        self.http
            .request_json(
                &format!("/agent/endpoints/{}/{}", safe_channel, safe_endpoint_value),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// List paused agent conversations in a group, newest first. A paused conversation is one
    /// a person took over; the agent stays out of it until it is resumed. Each pause carries
    /// `groupId`, `subgroupId`, `channel`, `endpointValue`, `contactIdentifier`, `reason`
    /// (`"human_reply"`, `"escalation"` or `"manual"`), `pausedBy`, `createdAt`, and `updatedAt`.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/pauses?...`
    /// `subgroup_id`, `channel` (`"sms"`), `endpoint_value` (the agent's number) and
    /// `contact_identifier` (the contact's number), digits only with no `+`, narrow the list.
    pub async fn get_agent_contact_pauses(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        channel: Option<&str>,
        endpoint_value: Option<&str>,
        contact_identifier: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
            ("channel", channel.map(|s| s.to_string())),
            ("endpointValue", endpoint_value.map(|s| s.to_string())),
            (
                "contactIdentifier",
                contact_identifier.map(|s| s.to_string()),
            ),
        ]);
        self.http
            .request_json(&format!("/agent/pauses{}", qs), Method::GET, None, options)
            .await
    }

    /// Pause an agent on one conversation (its number + one contact). The agent stops
    /// answering that contact on that number until the conversation is resumed; there is no
    /// timer. The conversation's pending replies are rejected. Pausing a paused conversation
    /// keeps the original pause. `channel` must be `"sms"`; `endpoint_value` and
    /// `contact_identifier` are 10 to 15 digits, country code included, no `+` (see
    /// [`PauseAgentConversationData`]). Returns the pause.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **PUT** `/agent/pauses/:channel/:endpointValue/:contactIdentifier`
    pub async fn pause_agent_conversation(
        &self,
        channel: &str,
        endpoint_value: &str,
        contact_identifier: &str,
        pause_data: &PauseAgentConversationData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("channel", Some(channel)),
            ("endpointValue", Some(endpoint_value)),
            ("contactIdentifier", Some(contact_identifier)),
        ])?;
        let safe_channel = urlencoding::encode(channel);
        let safe_endpoint_value = urlencoding::encode(endpoint_value);
        let safe_contact_identifier = urlencoding::encode(contact_identifier);
        let body = serde_json::to_value(pause_data)?;
        self.http
            .request_json(
                &format!(
                    "/agent/pauses/{}/{}/{}",
                    safe_channel, safe_endpoint_value, safe_contact_identifier
                ),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Resume an agent on a paused conversation, so it answers the contact's next text again.
    /// Returns the removed pause.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **DELETE** `/agent/pauses/:channel/:endpointValue/:contactIdentifier`
    pub async fn resume_agent_conversation(
        &self,
        channel: &str,
        endpoint_value: &str,
        contact_identifier: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("channel", Some(channel)),
            ("endpointValue", Some(endpoint_value)),
            ("contactIdentifier", Some(contact_identifier)),
        ])?;
        let safe_channel = urlencoding::encode(channel);
        let safe_endpoint_value = urlencoding::encode(endpoint_value);
        let safe_contact_identifier = urlencoding::encode(contact_identifier);
        self.http
            .request_json(
                &format!(
                    "/agent/pauses/{}/{}/{}",
                    safe_channel, safe_endpoint_value, safe_contact_identifier
                ),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// List held agent replies (the Pending Replies queue) in a group, newest first. Each
    /// draft carries `agentReplyDraftId`, `groupId`, `subgroupId`, `agentProfileId`,
    /// `conversationId`, `channel`, `endpointValue`, `contactPhoneNumber`, `inboundText`,
    /// `draftText`, `sentText`, `reason`, `status`, `decidedBy`, `decidedAt`, `sentMessageId`,
    /// `deliveredBy` (`"signalhouse"` or `"external"`; null until decided), `failureReason`
    /// (`"delivery_unconfirmed"` or null), `claimId` (the current claim's id; null until
    /// claimed), `createdAt`, and `updatedAt`.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/drafts?...`
    /// `subgroup_id`, `agent_profile_id` and `status` (`"pending"`, `"sending"`, `"sent"`,
    /// `"rejected"` or `"failed"`) narrow the list; `limit` defaults to 25, max 500.
    #[allow(clippy::too_many_arguments)]
    pub async fn get_agent_reply_drafts(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        agent_profile_id: Option<&str>,
        status: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
            ("agentProfileId", agent_profile_id.map(|s| s.to_string())),
            ("status", status.map(|s| s.to_string())),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/agent/drafts{}", qs), Method::GET, None, options)
            .await
    }

    /// Approve a held agent reply and send it from the agent's number, optionally with edited
    /// text (see [`ApproveAgentReplyDraftData`]). It goes through the normal SMS send path, so
    /// opt-out and billing apply; the draft comes back `"failed"` when the send path accepted
    /// nothing. A draft that is no longer pending returns 409.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/drafts/:agentReplyDraftId/approve`
    pub async fn approve_agent_reply_draft(
        &self,
        agent_reply_draft_id: &str,
        approval_data: &ApproveAgentReplyDraftData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentReplyDraftId", Some(agent_reply_draft_id))])?;
        let safe_id = urlencoding::encode(agent_reply_draft_id);
        let body = serde_json::to_value(approval_data)?;
        self.http
            .request_json(
                &format!("/agent/drafts/{}/approve", safe_id),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Reject a held agent reply so it is never sent. Returns the rejected draft. A draft that
    /// is no longer pending returns 409.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/drafts/:agentReplyDraftId/reject`
    pub async fn reject_agent_reply_draft(
        &self,
        agent_reply_draft_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentReplyDraftId", Some(agent_reply_draft_id))])?;
        let safe_id = urlencoding::encode(agent_reply_draft_id);
        self.http
            .request_json(
                &format!("/agent/drafts/{}/reject", safe_id),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Claim a held agent reply for delivery through your own channel instead of Signal
    /// House's send path, optionally with edited text (see [`ClaimAgentReplyDraftData`]).
    /// Nothing is sent: the draft becomes `"sending"` with `deliveredBy: "external"`,
    /// `sentText` and a new `claimId` set, and you then report the outcome with
    /// [`complete_agent_reply_draft`](Self::complete_agent_reply_draft) or
    /// [`release_agent_reply_draft`](Self::release_agent_reply_draft), passing that `claimId`
    /// back as `claim_id`. The claim means two
    /// approvals can never both deliver. A draft that is no longer pending returns 409; one
    /// whose number has left its subgroup returns 400 and is closed as failed.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/drafts/:agentReplyDraftId/claim`
    pub async fn claim_agent_reply_draft(
        &self,
        agent_reply_draft_id: &str,
        claim_data: &ClaimAgentReplyDraftData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentReplyDraftId", Some(agent_reply_draft_id))])?;
        let safe_id = urlencoding::encode(agent_reply_draft_id);
        let body = serde_json::to_value(claim_data)?;
        self.http
            .request_json(
                &format!("/agent/drafts/{}/claim", safe_id),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Mark a claimed agent reply sent once your channel accepted it, naming the claim with
    /// the required `claim_id` and optionally reporting your channel's message id (see
    /// [`CompleteAgentReplyDraftData`]). Only the claim named by `claim_id` can be completed;
    /// anything else returns 409. It is also accepted after an unfinished claim was closed as
    /// `"failed"` with `failureReason: "delivery_unconfirmed"`, which it turns into `"sent"`.
    /// Returns the draft, now sent.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/drafts/:agentReplyDraftId/complete`
    pub async fn complete_agent_reply_draft(
        &self,
        agent_reply_draft_id: &str,
        completion_data: &CompleteAgentReplyDraftData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentReplyDraftId", Some(agent_reply_draft_id)),
            ("claimId", Some(completion_data.claim_id.as_str())),
        ])?;
        let safe_id = urlencoding::encode(agent_reply_draft_id);
        let body = serde_json::to_value(completion_data)?;
        self.http
            .request_json(
                &format!("/agent/drafts/{}/complete", safe_id),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Give back a claim whose reply certainly did not go out: the draft returns to
    /// `"pending"`, or with `permanent` (see [`ReleaseAgentReplyDraftData`], which also carries
    /// the required `claim_id`) is closed as
    /// `"failed"`. If you cannot tell whether it went out, do not release it; a claim left
    /// unfinished for 30 minutes is closed as failed with `failureReason:
    /// "delivery_unconfirmed"`. Only the claim named by `claim_id` can be released; anything
    /// else returns 409.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **POST** `/agent/drafts/:agentReplyDraftId/release`
    pub async fn release_agent_reply_draft(
        &self,
        agent_reply_draft_id: &str,
        release_data: &ReleaseAgentReplyDraftData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[
            ("agentReplyDraftId", Some(agent_reply_draft_id)),
            ("claimId", Some(release_data.claim_id.as_str())),
        ])?;
        let safe_id = urlencoding::encode(agent_reply_draft_id);
        let body = serde_json::to_value(release_data)?;
        self.http
            .request_json(
                &format!("/agent/drafts/{}/release", safe_id),
                Method::POST,
                Some(&body),
                options,
            )
            .await
    }

    /// Publish an agent profile, setting `publishStatus` to `"published"` and stamping
    /// `publishedAt`. The server rejects the call with 400 when the agent has no enabled
    /// deployment, and with 409 when the agent is inactive.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **POST** `/agent/profiles/:agentProfileId/publish`
    pub async fn publish_agent_profile(
        &self,
        agent_profile_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentProfileId", Some(agent_profile_id))])?;
        let safe = urlencoding::encode(agent_profile_id);
        self.http
            .request_json(
                &format!("/agent/profiles/{}/publish", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Return an agent profile to draft (`publishStatus` `"draft"`).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **POST** `/agent/profiles/:agentProfileId/unpublish`
    pub async fn unpublish_agent_profile(
        &self,
        agent_profile_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentProfileId", Some(agent_profile_id))])?;
        let safe = urlencoding::encode(agent_profile_id);
        self.http
            .request_json(
                &format!("/agent/profiles/{}/unpublish", safe),
                Method::POST,
                None,
                options,
            )
            .await
    }

    /// Read the tenant AI settings for a scope (group, or a subgroup within it). Returns the
    /// scope's own row only — no group-level fallback.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/tenant-settings`
    pub async fn get_tenant_ai_settings(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/agent/tenant-settings{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create or update (upsert) the tenant AI settings for a scope. The scope is identified
    /// by the query params; the body carries the fields (see [`UpsertTenantAiSettingsData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/tenant-settings`
    pub async fn upsert_tenant_ai_settings(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        settings_data: &UpsertTenantAiSettingsData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
        ]);
        let body = serde_json::to_value(settings_data)?;
        self.http
            .request_json(
                &format!("/agent/tenant-settings{}", qs),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Delete the tenant AI settings for a scope.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/tenant-settings`
    pub async fn delete_tenant_ai_settings(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/agent/tenant-settings{}", qs),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// List the knowledge-base items an agent can answer from (RAG), scoped to a group and
    /// optionally narrowed to a subgroup and/or a single agent. Paginated (server default 25
    /// per page, max 100).
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/knowledge`
    pub async fn get_knowledge_items(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        agent_profile_id: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
            ("agentProfileId", agent_profile_id.map(|s| s.to_string())),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(
                &format!("/agent/knowledge{}", qs),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Get a single knowledge item by id.
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/knowledge/:knowledgeItemId`
    pub async fn get_knowledge_item(
        &self,
        knowledge_item_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("knowledgeItemId", Some(knowledge_item_id))])?;
        let safe = urlencoding::encode(knowledge_item_id);
        self.http
            .request_json(
                &format!("/agent/knowledge/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create a knowledge item (raw text stored; Atlas owns the embedding). `item_data` must
    /// include `group_id` and `text` (see [`CreateKnowledgeItemData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **POST** `/agent/knowledge`
    pub async fn create_knowledge_item(
        &self,
        item_data: &CreateKnowledgeItemData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = serde_json::to_value(item_data)?;
        self.http
            .request_json("/agent/knowledge", Method::POST, Some(&body), options)
            .await
    }

    /// Update a knowledge item. The item's scope is immutable (see [`UpdateKnowledgeItemData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/knowledge/:id`
    pub async fn update_knowledge_item(
        &self,
        id: &str,
        update_data: &UpdateKnowledgeItemData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = serde_json::to_value(update_data)?;
        self.http
            .request_json(
                &format!("/agent/knowledge/{}", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Delete a knowledge item by its id.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/knowledge/:id`
    pub async fn delete_knowledge_item(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/agent/knowledge/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }

    /// List the tools defined under a group (webhook / configured builtin / mcp), with
    /// webhookAuth secrets redacted. Paginated (server default 25 per page, max 100).
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/tools`
    #[allow(clippy::too_many_arguments)]
    pub async fn get_agent_tools(
        &self,
        group_id: &str,
        subgroup_id: Option<&str>,
        agent_profile_id: Option<&str>,
        tool_type: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("groupId", Some(group_id))])?;
        let qs = HttpClient::get_query_string(&[
            ("groupId", Some(group_id.to_string())),
            ("subgroupId", subgroup_id.map(|s| s.to_string())),
            ("agentProfileId", agent_profile_id.map(|s| s.to_string())),
            ("type", tool_type.map(|s| s.to_string())),
            ("page", page.map(|p| p.to_string())),
            ("limit", limit.map(|l| l.to_string())),
        ]);
        self.http
            .request_json(&format!("/agent/tools{}", qs), Method::GET, None, options)
            .await
    }

    /// Get a single agent tool by id (webhookAuth redacted).
    ///
    /// **Roles:** api, admin, developer, billing, user.
    ///
    /// **GET** `/agent/tools/:agentToolId`
    pub async fn get_agent_tool(
        &self,
        agent_tool_id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("agentToolId", Some(agent_tool_id))])?;
        let safe = urlencoding::encode(agent_tool_id);
        self.http
            .request_json(
                &format!("/agent/tools/{}", safe),
                Method::GET,
                None,
                options,
            )
            .await
    }

    /// Create an agent tool. A webhook tool's `webhook_url` must be a public https URL (see
    /// [`CreateAgentToolData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **POST** `/agent/tools`
    pub async fn create_agent_tool(
        &self,
        tool_data: &CreateAgentToolData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        let body = serde_json::to_value(tool_data)?;
        self.http
            .request_json("/agent/tools", Method::POST, Some(&body), options)
            .await
    }

    /// Update an agent tool. The tool's scope is immutable (see [`UpdateAgentToolData`]).
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **PUT** `/agent/tools/:id`
    pub async fn update_agent_tool(
        &self,
        id: &str,
        update_data: &UpdateAgentToolData,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        let body = serde_json::to_value(update_data)?;
        self.http
            .request_json(
                &format!("/agent/tools/{}", safe),
                Method::PUT,
                Some(&body),
                options,
            )
            .await
    }

    /// Delete an agent tool by its id.
    ///
    /// **Roles:** api, admin, developer.
    ///
    /// **DELETE** `/agent/tools/:id`
    pub async fn delete_agent_tool(
        &self,
        id: &str,
        options: Option<&RequestOptions>,
    ) -> Result<SignalHouseResponse, SignalHouseError> {
        HttpClient::require(&[("id", Some(id))])?;
        let safe = urlencoding::encode(id);
        self.http
            .request_json(
                &format!("/agent/tools/{}", safe),
                Method::DELETE,
                None,
                options,
            )
            .await
    }
}
