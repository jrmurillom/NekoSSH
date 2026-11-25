//! Puerto agnóstico de proveedor de IA (`trait AiProviderDriver`) y contratos de turno.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::schema::{AgentResponseKind, ConversationTurn};

/// Token cooperativo para abortar un turno de inferencia en vuelo.
#[derive(Clone, Default)]
pub struct TurnCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl TurnCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Estado de salud y disponibilidad del proveedor de IA activo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverHealthStatus {
    pub driver_id: String,
    pub available: bool,
    pub version: Option<String>,
    pub message: String,
}

/// Descriptor universal de un modelo disponible para el selector rápido en UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiModelDescriptor {
    pub id: String,
    pub label: String,
}

/// Errores tipados y accionables en español latino producidos por cualquier driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum AiDriverError {
    BinaryNotFound(String),
    AuthenticationRequired(String),
    Timeout(String),
    Cancelled,
    InvalidResponse(String),
    ExecutionFailed(String),
}

impl std::fmt::Display for AiDriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiDriverError::BinaryNotFound(msg) => write!(f, "{}", msg),
            AiDriverError::AuthenticationRequired(msg) => write!(f, "{}", msg),
            AiDriverError::Timeout(msg) => write!(f, "{}", msg),
            AiDriverError::Cancelled => write!(f, "Turno cancelado por el usuario."),
            AiDriverError::InvalidResponse(msg) => write!(f, "Respuesta inválida del modelo: {}", msg),
            AiDriverError::ExecutionFailed(msg) => write!(f, "Falla en ejecución del proveedor: {}", msg),
        }
    }
}

impl std::error::Error for AiDriverError {}

/// Solicitud universal enviada por el orquestador hacia cualquier `AiProviderDriver`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTurnRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub terminal_context: Option<String>,
    pub turns: Vec<ConversationTurn>,
    pub provider_conversation_id: Option<String>,
    pub selected_model: Option<String>,
    pub json_schema: Value,
    pub timeout_secs: u64,
}

/// Resultado estructurado devuelto por un `AiProviderDriver`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTurnOutcome {
    pub response: AgentResponseKind,
    pub provider_conversation_id: Option<String>,
    pub raw_model_used: Option<String>,
}

/// Puerto hexagonal para conectar motores de razonamiento (CLI o APIs REST) sin acoplar el Arnés.
pub trait AiProviderDriver: Send + Sync {
    /// Identificador canónico del proveedor (p. ej. `"agy"`, `"openai"`, `"ollama"`).
    fn driver_id(&self) -> &str;

    /// Verifica si el proveedor está instalado, accesible y listo para responder.
    fn probe_health(&self) -> DriverHealthStatus;

    /// Lista los modelos disponibles para poblar el selector rápido de modelos.
    fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiDriverError>;

    /// Ejecuta un turno de razonamiento respetando `cancel` y emitiendo progreso en tiempo real.
    fn execute_turn(
        &self,
        request: &AgentTurnRequest,
        cancel: &TurnCancellationToken,
        on_progress: &(dyn Fn(&str) + Send + Sync),
    ) -> Result<AgentTurnOutcome, AiDriverError>;
}
