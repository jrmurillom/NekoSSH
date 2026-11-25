//! Orquestador concurrente de turnos agénticos (`HarnessOrchestrator`) y cancelación en vivo.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ai::commands::HarnessRegistry;
use crate::ai::driver::{
    AgentTurnOutcome, AgentTurnRequest, AiDriverError, AiProviderDriver, TurnCancellationToken,
};
use crate::ai::fsm::{ExecutionMode, HarnessPhase};
use crate::ai::preferences::AiPreferencesDto;
use crate::ai::prompt::PromptAssembler;
use crate::ai::redactor::SecretRedactor;
use crate::ai::risk_guard::CommandRiskGuard;
use crate::ai::schema::{
    agent_turn_wire_json_schema, AgentResponseKind, ConversationRole, ConversationTurn,
    ConversationalPhaseTag,
};

/// Registro de tokens de cancelación para abortar procesos de IA activos por sesión o al salir.
#[derive(Clone, Default)]
pub struct AiTurnCancelRegistry(pub Arc<Mutex<HashMap<String, TurnCancellationToken>>>);

impl AiTurnCancelRegistry {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    /// Registra un nuevo token para `session_id`, cancelando cualquier turno previo de esa sesión.
    pub fn register(&self, session_id: &str) -> TurnCancellationToken {
        let token = TurnCancellationToken::new();
        let mut map = self.0.lock().unwrap();
        if let Some(prev) = map.insert(session_id.to_string(), token.clone()) {
            prev.cancel();
        }
        token
    }

    pub fn cancel_session(&self, session_id: &str) -> bool {
        let map = self.0.lock().unwrap();
        if let Some(token) = map.get(session_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn clear(&self, session_id: &str) {
        let mut map = self.0.lock().unwrap();
        map.remove(session_id);
    }

    pub fn cancel_all(&self) {
        let mut map = self.0.lock().unwrap();
        for (_, token) in map.drain() {
            token.cancel();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TurnExecutionResultDto {
    pub phase: HarnessPhase,
    pub outcome: AgentTurnOutcome,
    pub execution_mode: ExecutionMode,
    pub auto_approved_read_only: bool,
}

struct TurnSnapshot {
    generation: u64,
    phase: HarnessPhase,
    turns: Vec<ConversationTurn>,
    provider_conversation_id: Option<String>,
}

/// Coordina turno completo aplicando el patrón:
/// Snapshot bajo lock -> Unlock -> Redacción + Driver + Guardrails -> Relock con validación de época.
pub struct HarnessOrchestrator;

impl HarnessOrchestrator {
    pub fn execute_turn(
        session_id: &str,
        user_prompt: &str,
        terminal_context: Option<&str>,
        prefs: &AiPreferencesDto,
        registry: &HarnessRegistry,
        cancel_registry: &AiTurnCancelRegistry,
        driver: &dyn AiProviderDriver,
        on_progress: &(dyn Fn(&str) + Send + Sync),
    ) -> Result<TurnExecutionResultDto, AiDriverError> {
        let sanitized_user = SecretRedactor::redact(user_prompt.trim());
        if sanitized_user.is_empty() {
            return Err(AiDriverError::InvalidResponse(
                "La consulta del operador no puede estar vacía.".to_string(),
            ));
        }
        let sanitized_context = terminal_context
            .map(SecretRedactor::redact)
            .filter(|s| !s.trim().is_empty());

        // 1. Snapshot rápido bajo Mutex (si la sesión está Idle, abre Reconnaissance automáticamente)
        let snapshot = {
            let mut map = registry.0.lock().unwrap();
            let fsm = map
                .entry(session_id.to_string())
                .or_insert_with(|| {
                    crate::ai::fsm::AgentHarnessFsm::new(session_id, prefs.execution_mode)
                });
            fsm.set_mode(prefs.execution_mode);

            if matches!(fsm.phase(), HarnessPhase::Idle | HarnessPhase::Completed { .. }) {
                fsm.continue_conversation(sanitized_user.clone());
            } else if matches!(fsm.phase(), HarnessPhase::PausedOnDisconnect { .. }) {
                return Err(AiDriverError::ExecutionFailed(
                    "La sesión SSH está desconectada; reconecta antes de continuar el turno.".to_string(),
                ));
            }

            let same_driver = fsm.active_driver_id() == Some(driver.driver_id());
            let conv_id = if same_driver {
                fsm.provider_conversation_id().map(|s| s.to_string())
            } else {
                None
            };

            TurnSnapshot {
                generation: fsm.generation(),
                phase: fsm.phase().clone(),
                turns: fsm.turns().to_vec(),
                provider_conversation_id: conv_id,
            }
        }; // <-- Mutex liberado aquí antes de invocar al proceso/API externo

        // 2. Ensamblar instrucciones de 2 capas + contexto de fase actual
        let base_system = PromptAssembler::assemble_system_prompt(
            &prefs.operator_persona,
            &prefs.operator_infra_rules,
        );
        let phase_directive = build_phase_directive(&snapshot.phase);
        let full_system_prompt = format!("{}\n\n{}", base_system, phase_directive);

        let selected_model = {
            let m = prefs.selected_model.trim();
            if m.is_empty() {
                None
            } else {
                Some(m.to_string())
            }
        };

        let request = AgentTurnRequest {
            system_prompt: full_system_prompt,
            user_prompt: sanitized_user.clone(),
            terminal_context: sanitized_context,
            turns: snapshot.turns,
            provider_conversation_id: snapshot.provider_conversation_id,
            selected_model,
            json_schema: agent_turn_wire_json_schema(),
            timeout_secs: 120,
        };

        // 3. Ejecutar inferencia con token de cancelación registrado
        let cancel_token = cancel_registry.register(session_id);
        let driver_res = driver.execute_turn(&request, &cancel_token, on_progress);
        cancel_registry.clear(session_id);

        let mut outcome = driver_res?;

        // 4. Auditoría no bloqueante sobre la propuesta recibida (eleva riesgo y adjunta alerta, nunca aborta)
        let auto_approved_read_only =
            audit_and_annotate_response(&snapshot.phase, prefs.execution_mode, &mut outcome.response);

        // 5. Relock y Commit verificando que la época (generation) no haya cambiado
        let committed_phase = {
            let mut map = registry.0.lock().unwrap();
            let Some(fsm) = map.get_mut(session_id) else {
                return Err(AiDriverError::Cancelled);
            };
            if fsm.generation() != snapshot.generation || cancel_token.is_cancelled() {
                return Err(AiDriverError::Cancelled);
            }

            fsm.record_turn(ConversationRole::User, sanitized_user);
            fsm.record_turn(
                ConversationRole::Assistant,
                summarize_assistant_response(&outcome.response),
            );
            fsm.sync_provider_session(driver.driver_id(), outcome.provider_conversation_id.clone());

            match (&snapshot.phase, &outcome.response) {
                (HarnessPhase::Reconnaissance { .. }, AgentResponseKind::ProposePlan { tasks, .. }) => {
                    let _ = fsm.advance_to_plan(tasks.clone());
                }
                (HarnessPhase::PlanAndDebate { .. }, AgentResponseKind::ProposePlan { tasks, .. }) => {
                    let _ = fsm.update_plan_in_debate(tasks.clone());
                }
                (HarnessPhase::HaltedOnError { .. }, AgentResponseKind::ProposePlan { tasks, .. }) => {
                    let _ = fsm.replace_remaining_plan_from_error(tasks.clone());
                }
                (
                    HarnessPhase::Reconnaissance { .. },
                    AgentResponseKind::ReplyOnly {
                        phase_tag: Some(ConversationalPhaseTag::DirectAnswer),
                        ..
                    },
                ) => {
                    fsm.conclude_turn_preserving_memory();
                }
                _ => {}
            }

            fsm.phase().clone()
        };

        Ok(TurnExecutionResultDto {
            phase: committed_phase,
            outcome,
            execution_mode: prefs.execution_mode,
            auto_approved_read_only,
        })
    }
}

fn build_phase_directive(phase: &HarnessPhase) -> String {
    match phase {
        HarnessPhase::Reconnaissance { .. } => {
            "# CONTEXTO DE SESIÓN: CONVERSACIONAL PROGRESIVO (4 FASES)\nEvalúa la solicitud del operador. Si es consulta informativa y ya tienes evidencia, usa phase='direct_answer' y step=null. Si requieres telemetría, usa phase='diagnosis' con un comando de lectura pura en step. Si vas a proponer o ejecutar cambios, sigue Fase 2 (causa raíz + backup + rollback) y Fase 3 (un comando por turno con dry-run antes de recargar servicios) culminando con Fase 4 (Health Check).".to_string()
        }
        HarnessPhase::PlanAndDebate { .. } => {
            "# CONTEXTO DE SESIÓN: REVISIÓN DE PROPUESTA\nEl operador está evaluando la propuesta técnica. Responde con Markdown claro y step=null si es ajuste conceptual, o propón el primer paso en step.".to_string()
        }
        HarnessPhase::GuidedExecution {
            current_step_index,
            tasks,
            ..
        } => format!(
            "# CONTEXTO DE SESIÓN: EJECUCIÓN EN CURSO (Paso {}/{})\nAsiste al operador sobre el paso actual.",
            current_step_index + 1,
            tasks.len()
        ),
        HarnessPhase::HaltedOnError {
            step_index,
            tasks,
            failed_command,
            exit_code,
            stderr,
            ..
        } => format!(
            "# CONTEXTO DE SESIÓN: FALLO DETECTADO EN PASO {}/{}\nComando: `{}`\nCódigo de salida: {}\nSalida:\n{}\n\nExplica brevemente la causa en `message` y propón la corrección puntual en `step` (o indica si conviene rollback).",
            step_index + 1,
            tasks.len(),
            failed_command,
            exit_code,
            stderr
        ),
        _ => "# CONTEXTO DE SESIÓN: ASISTENTE CONVERSACIONAL".to_string(),
    }
}

/// Audita el comando deterministamente sin abortar jamás la respuesta.
/// Devuelve `true` solo si es lectura `🟢` verificada, elegible para auto-ejecución en `Modo Flexible`.
fn audit_and_annotate_response(
    phase: &HarnessPhase,
    mode: ExecutionMode,
    response: &mut AgentResponseKind,
) -> bool {
    match response {
        AgentResponseKind::ProposeStep {
            command,
            risk_level,
            audit_warning,
            ..
        } => {
            let report = CommandRiskGuard::evaluate_proposal(*risk_level, command);
            *risk_level = report.effective_risk;
            *audit_warning = report.audit_warning;

            // En Fase 3 formal pre-congelada o en Modo Estricto nunca se auto-ejecuta
            let blocked_by_phase = matches!(phase, HarnessPhase::GuidedExecution { .. });
            mode == ExecutionMode::Flexible && !blocked_by_phase && report.auto_runnable_in_flexible
        }
        AgentResponseKind::ProposePlan { tasks, .. } => {
            for task in tasks {
                task.risk_level = CommandRiskGuard::assess_and_elevate(task.risk_level, &task.command);
            }
            false
        }
        AgentResponseKind::ReplyOnly { .. } | AgentResponseKind::GoalReached { .. } => false,
    }
}

fn summarize_assistant_response(response: &AgentResponseKind) -> String {
    match response {
        AgentResponseKind::ProposeStep {
            reasoning,
            purpose,
            command,
            ..
        } => format!(
            "{} [Propuesto para {}: `{}`]",
            reasoning,
            purpose.as_deref().unwrap_or("inspección/acción"),
            command
        ),
        AgentResponseKind::ProposePlan { reasoning, tasks } => {
            format!("Propuesta de {} pasos: {}", tasks.len(), reasoning)
        }
        AgentResponseKind::ReplyOnly { message, .. } => message.clone(),
        AgentResponseKind::GoalReached { summary } => summary.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::drivers::FakeAiDriver;
    use crate::ai::schema::{AgentTaskProposal, RiskLevel};

    #[test]
    fn orquestador_sanitiza_contexto_y_auto_aprueba_solo_lectura_en_modo_flexible() {
        let registry = HarnessRegistry::new();
        let cancel_reg = AiTurnCancelRegistry::new();
        let driver = FakeAiDriver::new();
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ProposeStep {
                reasoning: "Ver puertos".to_string(),
                purpose: Some("listar puertos en escucha".to_string()),
                command: "ss -tulpn".to_string(),
                risk_level: RiskLevel::ReadOnly,
                backup_strategy: None,
                rollback_cmd: None,
                audit_warning: None,
                phase_tag: Some(ConversationalPhaseTag::Diagnosis),
            },
            provider_conversation_id: Some("conv-1".to_string()),
            raw_model_used: Some("fake-fast".to_string()),
        }));

        let prefs = AiPreferencesDto {
            execution_mode: ExecutionMode::Flexible,
            ..Default::default()
        };

        let result = HarnessOrchestrator::execute_turn(
            "term-flex",
            "Revisa Tomcat",
            Some("PASSWORD=super_secreto_123\nroot@vps:~#"),
            &prefs,
            &registry,
            &cancel_reg,
            &driver,
            &|_| {},
        )
        .expect("turn ok");

        assert!(result.auto_approved_read_only);
        assert!(matches!(result.phase, HarnessPhase::Reconnaissance { .. }));

        // Verificar que al driver nunca llegó el secreto en claro
        let sent = &driver.recorded_requests()[0];
        let ctx = sent.terminal_context.as_deref().unwrap();
        assert!(!ctx.contains("super_secreto_123"));
        assert!(ctx.contains("[REDACTED_SECRET]"));
    }

    #[test]
    fn orquestador_eleva_riesgo_y_adjunta_alerta_sin_bloquear_en_modo_flexible() {
        let registry = HarnessRegistry::new();
        let cancel_reg = AiTurnCancelRegistry::new();
        let driver = FakeAiDriver::new();
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ProposeStep {
                reasoning: "Instalar nginx en el servidor".to_string(),
                purpose: Some("instalar paquete nginx".to_string()),
                command: "apt-get install -y nginx".to_string(),
                risk_level: RiskLevel::ReadOnly, // IA subestima el riesgo
                backup_strategy: None,
                rollback_cmd: None,
                audit_warning: None,
                phase_tag: Some(ConversationalPhaseTag::Execution),
            },
            provider_conversation_id: None,
            raw_model_used: None,
        }));

        let prefs = AiPreferencesDto {
            execution_mode: ExecutionMode::Flexible,
            ..Default::default()
        };
        let res = HarnessOrchestrator::execute_turn(
            "term-nonblock",
            "Instala nginx",
            None,
            &prefs,
            &registry,
            &cancel_reg,
            &driver,
            &|_| {},
        )
        .expect("no debe abortar con error bloqueante");

        assert!(!res.auto_approved_read_only);
        match res.outcome.response {
            AgentResponseKind::ProposeStep {
                risk_level,
                audit_warning,
                ..
            } => {
                assert_eq!(risk_level, RiskLevel::ConfigChange);
                assert!(audit_warning.is_some());
            }
            other => panic!("Esperaba ProposeStep con alerta, llegó {:?}", other),
        }
    }

    #[test]
    fn orquestador_no_retiene_mutex_durante_inferencia_y_descarta_si_hubo_cancelacion() {
        let registry = HarnessRegistry::new();
        let cancel_reg = AiTurnCancelRegistry::new();
        let driver = FakeAiDriver::new().with_delay_ms(120);
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ReplyOnly {
                message: "Respuesta tardía".to_string(),
                phase_tag: Some(ConversationalPhaseTag::DirectAnswer),
            },
            provider_conversation_id: None,
            raw_model_used: None,
        }));

        let prefs = AiPreferencesDto::default();
        let reg_clone = registry.clone();
        let cancel_clone = cancel_reg.clone();

        let handle = std::thread::spawn(move || {
            HarnessOrchestrator::execute_turn(
                "term-concurrent",
                "Analiza",
                None,
                &prefs,
                &reg_clone,
                &cancel_clone,
                &driver,
                &|_| {},
            )
        });

        // Esperar brevemente a que entre en inferencia y verificar que el Mutex NO está bloqueado
        std::thread::sleep(std::time::Duration::from_millis(30));
        let phase_during_turn = registry.with_or_create("term-concurrent", ExecutionMode::Strict, |fsm| {
            fsm.phase().clone()
        });
        assert!(matches!(phase_during_turn, HarnessPhase::Reconnaissance { .. }));

        // Cancelar el turno activo sin bloqueo
        assert!(cancel_reg.cancel_session("term-concurrent"));

        let res = handle.join().unwrap();
        assert_eq!(res.unwrap_err(), AiDriverError::Cancelled);
    }

    #[test]
    fn orquestador_transiciona_a_plan_and_debate_al_recibir_plan_valido() {
        let registry = HarnessRegistry::new();
        let cancel_reg = AiTurnCancelRegistry::new();
        let driver = FakeAiDriver::new();
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ProposePlan {
                reasoning: "Plan de instalación Tomcat".to_string(),
                tasks: vec![AgentTaskProposal::new(
                    1,
                    "Instalar Java",
                    "Dependencia",
                    "apt-get install -y openjdk-21-jre-headless",
                    RiskLevel::ReadOnly, // Será elevado por CommandRiskGuard a ConfigChange
                    "dpkg --get-selections",
                    "apt-get purge -y openjdk-21-jre-headless",
                )],
            },
            provider_conversation_id: Some("conv-plan".to_string()),
            raw_model_used: Some("fake-pro".to_string()),
        }));

        let prefs = AiPreferencesDto::default();
        let res = HarnessOrchestrator::execute_turn(
            "term-plan",
            "Instalar Tomcat",
            None,
            &prefs,
            &registry,
            &cancel_reg,
            &driver,
            &|_| {},
        )
        .expect("plan ok");

        match res.phase {
            HarnessPhase::PlanAndDebate { plan, .. } => {
                assert_eq!(plan.len(), 1);
                // Verificamos que CommandRiskGuard elevó de ReadOnly a ConfigChange
                assert_eq!(plan[0].risk_level, RiskLevel::ConfigChange);
            }
            other => panic!("Esperaba PlanAndDebate, llegó {:?}", other),
        }
    }

    #[test]
    fn orquestador_inyecta_error_en_halted_y_reemplaza_cola_pendiente_sin_auto_aprobar() {
        let registry = HarnessRegistry::new();
        registry.with_or_create("term-halt", ExecutionMode::Flexible, |fsm| {
            fsm.start_recon("Instalar stack").unwrap();
            fsm.advance_to_plan(vec![
                AgentTaskProposal::new(
                    1,
                    "Paso 1",
                    "Base",
                    "mkdir -p /opt/app",
                    RiskLevel::ConfigChange,
                    "ls /opt",
                    "rmdir /opt/app",
                ),
                AgentTaskProposal::new(
                    2,
                    "Paso 2",
                    "Iniciar",
                    "systemctl start app-old",
                    RiskLevel::ServiceImpact,
                    "systemctl status app-old",
                    "systemctl stop app-old",
                ),
            ])
            .unwrap();
            fsm.approve_plan_and_start_execution().unwrap();
            fsm.complete_current_step().unwrap();
            fsm.halt_on_error("systemctl start app-old", 5, "Unit app-old.service not found")
                .unwrap();
        });

        let cancel_reg = AiTurnCancelRegistry::new();
        let driver = FakeAiDriver::new();
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ProposePlan {
                reasoning: "Corregir nombre de servicio en el resto del plan".to_string(),
                tasks: vec![AgentTaskProposal::new(
                    1,
                    "Iniciar unidad real",
                    "El servicio se llama app",
                    "systemctl start app",
                    RiskLevel::ServiceImpact,
                    "systemctl status app",
                    "systemctl stop app",
                )],
            },
            provider_conversation_id: Some("conv-halt".to_string()),
            raw_model_used: Some("fake-pro".to_string()),
        }));

        let prefs = AiPreferencesDto {
            execution_mode: ExecutionMode::Flexible,
            ..Default::default()
        };
        let res = HarnessOrchestrator::execute_turn(
            "term-halt",
            "Corrige el paso fallido",
            None,
            &prefs,
            &registry,
            &cancel_reg,
            &driver,
            &|_| {},
        )
        .expect("turn ok");

        assert!(!res.auto_approved_read_only);
        let sent_sys = &driver.recorded_requests()[0].system_prompt;
        assert!(sent_sys.contains("Unit app-old.service not found"));

        match res.phase {
            HarnessPhase::GuidedExecution {
                tasks,
                current_step_index,
                ..
            } => {
                assert_eq!(current_step_index, 1);
                assert_eq!(tasks.len(), 2);
                assert!(tasks[0].is_done);
                assert_eq!(tasks[1].command, "systemctl start app");
            }
            other => panic!("Esperaba GuidedExecution restaurado, llegó {:?}", other),
        }
    }
}
