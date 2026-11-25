//! Módulo de inteligencia artificial y copiloto agéntico determinista para NekoSSH.

pub mod fsm;
pub mod schema;
pub mod risk_guard;
pub mod redactor;
pub mod prompt;
pub mod preferences;
pub mod driver;
pub mod drivers;
pub mod orchestrator;
pub mod pty_tap;
pub mod commands;

pub use fsm::{AgentHarnessFsm, ExecutionMode, HarnessPhase, HarnessTransitionError};
pub use schema::{
    agent_turn_wire_json_schema, AgentResponseKind, AgentTaskProposal, AgentTurnWireEnvelope,
    ConversationRole, ConversationTurn, ConversationalPhaseTag, RiskLevel,
};
pub use risk_guard::{CommandRiskGuard, ProposalSafetyReport, ReadOnlyEnforcer};
pub use redactor::SecretRedactor;
pub use prompt::{PromptAssembler, DEFAULT_OPERATOR_INFRA_RULES, DEFAULT_OPERATOR_PERSONA};
pub use preferences::AiPreferencesDto;
pub use driver::{
    AgentTurnOutcome, AgentTurnRequest, AiDriverError, AiModelDescriptor, AiProviderDriver,
    DriverHealthStatus, TurnCancellationToken,
};
pub use orchestrator::{AiTurnCancelRegistry, HarnessOrchestrator, TurnExecutionResultDto};
pub use pty_tap::{
    apply_sentinel_completion_to_fsm, PtyTapRegistry, SentinelCompletion, SentinelPurpose,
};
pub use commands::{
    ActiveShellsCatalog, AiFeedEntryDto, AiSelectionQueue, AiUiFeedRegistry, HarnessRegistry,
};
