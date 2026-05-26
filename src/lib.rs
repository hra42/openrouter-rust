#![doc = include_str!("../docs/recipes/quickstart.md")]
//!
//! # More recipes
//!
//! - [Streaming](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/streaming.md)
//! - [Tools](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/tools.md)
//! - [Structured outputs](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/structured_outputs.md)
//! - [Multimodal](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/multimodal.md)
//! - [Provider routing](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/provider_routing.md)
//! - [ZDR](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/zdr.md)
//! - [Key management](https://github.com/hra42/openrouter-rust/blob/main/docs/recipes/key_management.md)

#![allow(clippy::result_large_err)]
#![deny(missing_docs)]

pub mod client;
pub mod error;
pub mod mcp;
pub mod oauth;
mod request;
#[cfg(feature = "beta")]
pub mod responses;
pub mod retry;
pub mod stream;
pub mod tool_call_accumulator;
pub mod types;
pub mod webhooks;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use retry::RetryConfig;
pub use stream::EventStream;
pub use tool_call_accumulator::ToolCallAccumulator;
pub use types::{
    create_file_parser_plugin, create_user_message_with_audio,
    create_user_message_with_audio_bytes, create_user_message_with_base64_image,
    create_user_message_with_base64_image_bytes, create_user_message_with_base64_pdf,
    create_user_message_with_files, create_user_message_with_image,
    create_user_message_with_image_detail, create_user_message_with_images,
    create_user_message_with_pdf, create_user_message_with_text_content,
    create_user_message_with_text_file, create_user_message_with_text_files,
    encode_image_bytes_to_base64, encode_image_to_base64, ActivityData, ActivityOptions,
    ActivityResponse, Annotation, ApiKey, AudioFormat, ChatCompletionRequest,
    ChatCompletionResponse, Choice, CompletionRequest, CompletionResponse, Content, ContentBuilder,
    ContentPart, CreateKeyRequest, CreateKeyResponse, CreditsData, CreditsResponse, DeleteKeyData,
    DeleteKeyResponse, Delta, File, FileAnnotation, FileParserEngine, FilePdfConfig,
    FilePluginConfig, FileRef, FunctionCall, FunctionDef, GetKeyByHashResponse, ImageDetail,
    ImageUrl, InputAudio, KeyData, KeyRateLimit, KeyResponse, ListKeysOptions, ListKeysResponse,
    ListModelsOptions, Message, Model, ModelArchitecture, ModelDefaultParameters, ModelEndpoint,
    ModelEndpointPricing, ModelEndpointsArchitecture, ModelEndpointsData, ModelEndpointsResponse,
    ModelPerRequestLimits, ModelPricing, ModelTopProvider, ModelsResponse, Plugin, Provider,
    ProviderInfo, ProvidersResponse, ReasoningConfig, ResponseFormat, Role, Tool, ToolCall,
    ToolChoice, UpdateKeyRequest, UpdateKeyResponse, UrlCitation, Usage, WebPluginConfig,
};
pub use types::{
    AssignKeysRequest, AssignKeysResponse, AssignMembersRequest, AssignMembersResponse,
    BulkAddWorkspaceMembersResponse, BulkRemoveWorkspaceMembersResponse, CreateGuardrailRequest,
    CreateWorkspaceRequest, CreateWorkspaceResponse, DeleteGuardrailResponse,
    DeleteWorkspaceResponse, GetWorkspaceResponse, Guardrail, GuardrailKeyAssignment,
    GuardrailMemberAssignment, ListGuardrailKeyAssignmentsResponse,
    ListGuardrailMemberAssignmentsResponse, ListGuardrailsOptions, ListGuardrailsResponse,
    ListOrganizationMembersOptions, ListOrganizationMembersResponse, ListWorkspacesOptions,
    ListWorkspacesResponse, OrganizationMember, OrganizationMemberRole, PercentileStats,
    PublicEndpoint, PublicEndpointPricing, RerankDocument, RerankRequest, RerankResponse,
    RerankResult, RerankUsage, ResetInterval, SpeechFormat, SpeechProvider, SpeechRequest,
    SpeechResponse, UpdateGuardrailRequest, UpdateWorkspaceRequest, UpdateWorkspaceResponse,
    VideoAspectRatio, VideoContentPartImage, VideoContentResponse, VideoFrameImage, VideoFrameType,
    VideoGenerationRequest, VideoGenerationResponse, VideoGenerationUsage, VideoImageUrl,
    VideoModel, VideoModelsResponse, VideoProvider, VideoResolution, VideoStatus, Workspace,
    WorkspaceMember, WorkspaceMemberRole, ZdrEndpointsResponse,
};
