//! Shared serde types for messages, requests, and responses.

mod account;
mod common;
mod discovery;
mod guardrails;
mod message;
mod multimodal;
mod organization;
mod request;
mod response;
mod workspace;

pub use account::{
    ActivityData, ActivityOptions, ActivityResponse, ApiKey, CreateKeyRequest, CreateKeyResponse,
    CreditsData, CreditsResponse, DeleteKeyData, DeleteKeyResponse, GetKeyByHashResponse, KeyData,
    KeyRateLimit, KeyResponse, ListKeysOptions, ListKeysResponse, UpdateKeyRequest,
    UpdateKeyResponse,
};
pub use common::{
    Annotation, FileAnnotation, FilePdfConfig, FilePluginConfig, FunctionCall, FunctionDef,
    JsonSchema, Plugin, Provider, ReasoningConfig, ResponseFormat, Tool, ToolCall, ToolChoice,
    UrlCitation, WebPluginConfig,
};
pub use discovery::{
    ListModelsOptions, Model, ModelArchitecture, ModelDefaultParameters, ModelEndpoint,
    ModelEndpointPricing, ModelEndpointsArchitecture, ModelEndpointsData, ModelEndpointsResponse,
    ModelPerRequestLimits, ModelPricing, ModelTopProvider, ModelsResponse, ProviderInfo,
    ProvidersResponse,
};
pub use guardrails::{
    AssignKeysRequest, AssignKeysResponse, AssignMembersRequest, AssignMembersResponse,
    CreateGuardrailRequest, DeleteGuardrailResponse, Guardrail, GuardrailKeyAssignment,
    GuardrailMemberAssignment, ListGuardrailKeyAssignmentsResponse,
    ListGuardrailMemberAssignmentsResponse, ListGuardrailsOptions, ListGuardrailsResponse,
    PercentileStats, PublicEndpoint, PublicEndpointPricing, ResetInterval, UpdateGuardrailRequest,
    ZdrEndpointsResponse,
};
pub use message::{Content, ContentPart, FileRef, ImageUrl, InputAudio, Message, Role};
pub use multimodal::{
    create_file_parser_plugin, create_user_message_with_audio,
    create_user_message_with_audio_bytes, create_user_message_with_base64_image,
    create_user_message_with_base64_image_bytes, create_user_message_with_base64_pdf,
    create_user_message_with_files, create_user_message_with_image,
    create_user_message_with_image_detail, create_user_message_with_images,
    create_user_message_with_pdf, create_user_message_with_text_content,
    create_user_message_with_text_file, create_user_message_with_text_files,
    encode_image_bytes_to_base64, encode_image_to_base64, AudioFormat, ContentBuilder, File,
    FileParserEngine, ImageDetail,
};
pub use organization::{
    ListOrganizationMembersOptions, ListOrganizationMembersResponse, OrganizationMember,
    OrganizationMemberRole,
};
pub use request::{ChatCompletionRequest, CompletionRequest};
pub use response::{
    ChatCompletionResponse, Choice, CompletionChoice, CompletionResponse, Delta, LogProbs,
    TokenDetails, Usage,
};
pub(crate) use workspace::BulkWorkspaceMembersRequest;
pub use workspace::{
    BulkAddWorkspaceMembersResponse, BulkRemoveWorkspaceMembersResponse, CreateWorkspaceRequest,
    CreateWorkspaceResponse, DeleteWorkspaceResponse, GetWorkspaceResponse, ListWorkspacesOptions,
    ListWorkspacesResponse, UpdateWorkspaceRequest, UpdateWorkspaceResponse, Workspace,
    WorkspaceMember, WorkspaceMemberRole,
};
