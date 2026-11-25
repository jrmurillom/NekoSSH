//! Esquemas tipados, sobre conversacional resiliente v2 y contratos de datos para NekoAI.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Solo lectura: consulta de estado, estadísticas o diagnóstico sin efectos secundarios.
    ReadOnly,
    /// Mutación de archivos, directorios o paquetes sin detener servicios.
    ConfigChange,
    /// Afectación a demonios, puertos en escucha o reglas de red en vivo.
    ServiceImpact,
    /// Acciones potencialmente destructivas, borrados masivos o reinicios de sistema.
    Destructive,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::ReadOnly => "read_only",
            RiskLevel::ConfigChange => "config_change",
            RiskLevel::ServiceImpact => "service_impact",
            RiskLevel::Destructive => "destructive",
        }
    }

    pub fn is_mutating(&self) -> bool {
        !matches!(self, RiskLevel::ReadOnly)
    }
}

/// Etapa operativa declarada por el modelo dentro del ciclo adaptativo de 4 fases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConversationalPhaseTag {
    #[default]
    Diagnosis,
    Proposal,
    Execution,
    Verification,
    DirectAnswer,
}

/// Rol de cada mensaje en el historial portátil de conversación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationRole {
    User,
    Assistant,
    SystemEvent,
}

/// Turno individual dentro de la bitácora portátil multi-proveedor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: ConversationRole,
    pub content: String,
}

/// Propuesta individual de tarea dentro de un plan estructurado (retrocompatible).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTaskProposal {
    pub id: u32,
    pub title: String,
    pub why: String,
    pub command: String,
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub backup_strategy: String,
    #[serde(default)]
    pub rollback_cmd: String,
    #[serde(default)]
    pub is_done: bool,
    #[serde(default)]
    pub is_rolled_back: bool,
}

impl AgentTaskProposal {
    pub fn new(
        id: u32,
        title: impl Into<String>,
        why: impl Into<String>,
        command: impl Into<String>,
        risk_level: RiskLevel,
        backup_strategy: impl Into<String>,
        rollback_cmd: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            why: why.into(),
            command: command.into(),
            risk_level,
            backup_strategy: backup_strategy.into(),
            rollback_cmd: rollback_cmd.into(),
            is_done: false,
            is_rolled_back: false,
        }
    }

    /// Valida que la tarea contenga justificación técnica y contingencia cuando aplique.
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("El título de la tarea no puede estar vacío".to_string());
        }
        if self.command.trim().is_empty() {
            return Err("El comando de la tarea no puede estar vacío".to_string());
        }
        if self.why.trim().is_empty() {
            return Err("La justificación técnica (why) es obligatoria".to_string());
        }
        if self.risk_level.is_mutating() {
            if self.backup_strategy.trim().is_empty() {
                return Err(format!(
                    "La tarea #{} modifica el sistema ({}) y exige definir backup_strategy",
                    self.id,
                    self.risk_level.as_str()
                ));
            }
            if self.rollback_cmd.trim().is_empty() {
                return Err(format!(
                    "La tarea #{} modifica el sistema ({}) y exige definir rollback_cmd",
                    self.id,
                    self.risk_level.as_str()
                ));
            }
        }
        Ok(())
    }
}

/// Tipo discriminado interno de respuesta validada del asistente agéntico.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AgentResponseKind {
    /// Respuesta rica + propuesta de UN comando ejecutable por turno.
    ProposeStep {
        reasoning: String,
        #[serde(default)]
        purpose: Option<String>,
        command: String,
        risk_level: RiskLevel,
        backup_strategy: Option<String>,
        rollback_cmd: Option<String>,
        #[serde(default)]
        audit_warning: Option<String>,
        #[serde(default)]
        phase_tag: Option<ConversationalPhaseTag>,
    },
    /// Propuesta completa de plan estructurado por tareas (retrocompatibilidad).
    ProposePlan {
        reasoning: String,
        tasks: Vec<AgentTaskProposal>,
    },
    /// Respuesta conversacional pura (causa raíz, diagnóstico final, debate o veredicto Fase 4).
    ReplyOnly {
        message: String,
        #[serde(default)]
        phase_tag: Option<ConversationalPhaseTag>,
    },
    /// Indicación explícita de objetivo completado (conserva la memoria de sesión).
    GoalReached {
        summary: String,
    },
}

/// Carga de paso individual dentro del sobre plano de transporte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireStepPayload {
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub reasoning: Option<String>,
    pub command: String,
    #[serde(default)]
    pub risk_level: Option<RiskLevel>,
    #[serde(default)]
    pub backup_strategy: Option<String>,
    #[serde(default)]
    pub rollback_cmd: Option<String>,
}

/// Sobre de transporte JSON resiliente v2 (y compatible con formato previo `kind`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTurnWireEnvelope {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub phase: Option<ConversationalPhaseTag>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub step: Option<WireStepPayload>,
    #[serde(default)]
    pub plan: Option<Vec<AgentTaskProposal>>,
}

impl TryFrom<AgentTurnWireEnvelope> for AgentResponseKind {
    type Error = String;

    fn try_from(env: AgentTurnWireEnvelope) -> Result<Self, Self::Error> {
        let rich_message = env
            .message
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // 1. Formato v2 principal: determinado por presencia de `step` o `message` (sin requerir `kind`)
        let effective_kind = env
            .kind
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                if env.step.as_ref().map(|s| !s.command.trim().is_empty()).unwrap_or(false) {
                    "propose_step"
                } else if env.plan.as_ref().map(|p| !p.is_empty()).unwrap_or(false) {
                    "propose_plan"
                } else {
                    "reply_only"
                }
            });

        match effective_kind {
            "propose_step" => {
                let Some(step) = env.step else {
                    // Degradación elegante: si vino `message` sin `step`, responder conversacionalmente
                    if let Some(msg) = rich_message {
                        return Ok(AgentResponseKind::ReplyOnly {
                            message: msg,
                            phase_tag: env.phase,
                        });
                    }
                    return Err("Respuesta sin 'step' ni 'message'".to_string());
                };
                let cmd = step.command.trim().to_string();
                if cmd.is_empty() {
                    if let Some(msg) = rich_message {
                        return Ok(AgentResponseKind::ReplyOnly {
                            message: msg,
                            phase_tag: env.phase,
                        });
                    }
                    return Err("Respuesta 'propose_step' con comando vacío y sin mensaje".to_string());
                }

                let step_reasoning = step
                    .reasoning
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                let purpose = step
                    .purpose
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .or_else(|| step_reasoning.clone());

                let reasoning = rich_message
                    .or(step_reasoning)
                    .or_else(|| purpose.clone())
                    .unwrap_or_else(|| "Comando propuesto para continuar la intervención.".to_string());

                Ok(AgentResponseKind::ProposeStep {
                    reasoning,
                    purpose,
                    command: cmd,
                    risk_level: step.risk_level.unwrap_or(RiskLevel::ReadOnly),
                    backup_strategy: step.backup_strategy.filter(|s| !s.trim().is_empty()),
                    rollback_cmd: step.rollback_cmd.filter(|s| !s.trim().is_empty()),
                    audit_warning: None,
                    phase_tag: env.phase.or(Some(ConversationalPhaseTag::Diagnosis)),
                })
            }
            "propose_plan" => {
                let Some(tasks) = env.plan.filter(|t| !t.is_empty()) else {
                    if let Some(msg) = rich_message {
                        return Ok(AgentResponseKind::ReplyOnly {
                            message: msg,
                            phase_tag: env.phase,
                        });
                    }
                    return Err("Respuesta 'propose_plan' vacía".to_string());
                };
                let reasoning = rich_message
                    .unwrap_or_else(|| "Plan estructurado propuesto para revisión.".to_string());
                Ok(AgentResponseKind::ProposePlan { reasoning, tasks })
            }
            "goal_reached" => {
                let summary = rich_message
                    .unwrap_or_else(|| "Objetivo concluido.".to_string());
                Ok(AgentResponseKind::ReplyOnly {
                    message: summary,
                    phase_tag: Some(ConversationalPhaseTag::DirectAnswer),
                })
            }
            _ => {
                let msg = rich_message
                    .unwrap_or_else(|| "No se recibió contenido adicional del asistente.".to_string());
                Ok(AgentResponseKind::ReplyOnly {
                    message: msg,
                    phase_tag: env.phase.or(Some(ConversationalPhaseTag::DirectAnswer)),
                })
            }
        }
    }
}

/// Esquema JSON plano v2 (Draft-07 sin `oneOf`) enfocado en explicación Markdown rica + 1 comando por turno.
pub fn agent_turn_wire_json_schema() -> Value {
    json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "additionalProperties": false,
        "required": ["phase", "message"],
        "properties": {
            "phase": {
                "type": "string",
                "enum": ["diagnosis", "proposal", "execution", "verification", "direct_answer"],
                "description": "Etapa operativa actual de las 4 fases, o direct_answer para respuesta final/directa."
            },
            "message": {
                "type": "string",
                "description": "Explicación técnica clara y detallada en Markdown (hallazgos, causa raíz, tabla resumen, riesgos, respaldo o veredicto)."
            },
            "step": {
                "type": ["object", "null"],
                "additionalProperties": false,
                "required": ["purpose", "command", "risk_level"],
                "properties": {
                    "purpose": {
                        "type": "string",
                        "description": "Frase corta en infinitivo explicando para qué sirve el comando (ej. 'identificar la distribución Linux')."
                    },
                    "command": {
                        "type": "string",
                        "description": "Un único comando shell no interactivo listo para ejecutar en PTY."
                    },
                    "risk_level": {
                        "type": "string",
                        "enum": ["read_only", "config_change", "service_impact", "destructive"]
                    },
                    "backup_strategy": { "type": ["string", "null"] },
                    "rollback_cmd": { "type": ["string", "null"] }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarea_solo_lectura_valida_sin_rollback() {
        let task = AgentTaskProposal::new(
            1,
            "Revisar memoria",
            "Verificar si hay RAM suficiente",
            "free -h",
            RiskLevel::ReadOnly,
            "",
            "",
        );
        assert!(task.validate().is_ok());
    }

    #[test]
    fn sobre_v2_convierte_propuesta_con_purpose_y_mensaje_sin_bloquear_campos_opcionales() {
        let valid_v2 = r#"{
            "phase": "diagnosis",
            "message": "Voy a inspeccionar la distribución y versión del kernel antes de sugerir cambios.",
            "step": {
                "purpose": "identificar la distribución Linux y el kernel instalado",
                "command": "cat /etc/os-release 2>/dev/null || uname -a",
                "risk_level": "read_only",
                "backup_strategy": null,
                "rollback_cmd": null
            }
        }"#;
        let env: AgentTurnWireEnvelope = serde_json::from_str(valid_v2).expect("env v2");
        let parsed = AgentResponseKind::try_from(env).expect("convert");
        match parsed {
            AgentResponseKind::ProposeStep {
                purpose,
                command,
                phase_tag,
                ..
            } => {
                assert_eq!(
                    purpose.as_deref(),
                    Some("identificar la distribución Linux y el kernel instalado")
                );
                assert_eq!(command, "cat /etc/os-release 2>/dev/null || uname -a");
                assert_eq!(phase_tag, Some(ConversationalPhaseTag::Diagnosis));
            }
            other => panic!("Esperaba ProposeStep, llegó {:?}", other),
        }

        // Un paso mutante sin rollback ya NO aborta el parseo (se delegará como audit_warning en tarjeta)
        let partial_mutating = r#"{
            "phase": "execution",
            "message": "Recargar nginx",
            "step": {
                "purpose": "recargar configuración de nginx",
                "command": "systemctl reload nginx",
                "risk_level": "service_impact"
            }
        }"#;
        let env_mut: AgentTurnWireEnvelope = serde_json::from_str(partial_mutating).expect("mut env");
        assert!(matches!(
            AgentResponseKind::try_from(env_mut).expect("degradado limpio"),
            AgentResponseKind::ProposeStep { .. }
        ));

        let schema = agent_turn_wire_json_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema.get("oneOf").is_none());
    }
}
