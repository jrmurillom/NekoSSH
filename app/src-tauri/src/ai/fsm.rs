//! Máquina de estados finita determinista para el arnés de IA (`AgentHarnessFsm`).

use serde::{Deserialize, Serialize};
use super::risk_guard::CommandRiskGuard;
use super::schema::{AgentTaskProposal, ConversationRole, ConversationTurn, RiskLevel};

const MAX_PORTABLE_TURNS: usize = 40;

/// Modos de supervisión operativa del arnés.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// Modo predeterminado: autorización humana obligatoria para absolutamente todo.
    #[default]
    Strict,
    /// Modo semi-supervisado: autoriza automáticamente comandos de lectura pura (🟢)
    /// pero frena obligatoriamente ante configuración, servicios o riesgo destructivo.
    Flexible,
}

impl ExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::Strict => "strict",
            ExecutionMode::Flexible => "flexible",
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "flexible" => ExecutionMode::Flexible,
            _ => ExecutionMode::Strict,
        }
    }
}

/// Fases y estados formales del ciclo de vida del arnés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HarnessPhase {
    /// Sin misión en curso. Escucha pasiva apagada.
    Idle,
    /// Fase 1: Diagnóstico previo y recolección de evidencia en solo lectura.
    Reconnaissance {
        objective: String,
    },
    /// Fase 2: Presentación del plan estructurado y debate interactivo sin ejecución.
    PlanAndDebate {
        objective: String,
        plan: Vec<AgentTaskProposal>,
    },
    /// Fase 3: Ejecución guiada por tareas con autorización paso a paso.
    GuidedExecution {
        objective: String,
        tasks: Vec<AgentTaskProposal>,
        current_step_index: usize,
    },
    /// Freno determinista tras la falla de una tarea (`exit_code != 0`), conservando el plan intacto.
    HaltedOnError {
        objective: String,
        tasks: Vec<AgentTaskProposal>,
        step_index: usize,
        failed_command: String,
        exit_code: i32,
        stderr: String,
    },
    /// Estado congelado por pérdida de conectividad o reinicio de terminal.
    PausedOnDisconnect {
        previous_phase: Box<HarnessPhase>,
    },
    /// Misión concluida con éxito total de las tareas del plan.
    Completed {
        objective: String,
        completed_tasks_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessTransitionError(pub String);

impl std::fmt::Display for HarnessTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error de transición en arnés: {}", self.0)
    }
}

impl std::error::Error for HarnessTransitionError {}

/// Instancia de arnés asociada a un contexto de sesión de terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHarnessFsm {
    session_id: String,
    mode: ExecutionMode,
    phase: HarnessPhase,
    turns: Vec<ConversationTurn>,
    provider_conversation_id: Option<String>,
    active_driver_id: Option<String>,
    generation: u64,
}

impl AgentHarnessFsm {
    pub fn new(session_id: impl Into<String>, mode: ExecutionMode) -> Self {
        Self {
            session_id: session_id.into(),
            mode,
            phase: HarnessPhase::Idle,
            turns: Vec::new(),
            provider_conversation_id: None,
            active_driver_id: None,
            generation: 0,
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn mode(&self) -> ExecutionMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ExecutionMode) {
        self.mode = mode;
    }

    pub fn phase(&self) -> &HarnessPhase {
        &self.phase
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn turns(&self) -> &[ConversationTurn] {
        &self.turns
    }

    pub fn provider_conversation_id(&self) -> Option<&str> {
        self.provider_conversation_id.as_deref()
    }

    pub fn active_driver_id(&self) -> Option<&str> {
        self.active_driver_id.as_deref()
    }

    /// Registra un turno en la bitácora portátil con límite acotado de memoria.
    pub fn record_turn(&mut self, role: ConversationRole, content: impl Into<String>) {
        let text = content.into();
        if text.trim().is_empty() {
            return;
        }
        self.turns.push(ConversationTurn { role, content: text });
        if self.turns.len() > MAX_PORTABLE_TURNS {
            let overflow = self.turns.len() - MAX_PORTABLE_TURNS;
            self.turns.drain(0..overflow);
        }
    }

    /// Sincroniza el identificador de conversación nativa del proveedor.
    /// Si cambia de proveedor, invalida el ID previo para evitar colisión entre drivers.
    pub fn sync_provider_session(
        &mut self,
        driver_id: &str,
        new_conversation_id: Option<String>,
    ) {
        let changed_driver = self
            .active_driver_id
            .as_deref()
            .map(|prev| prev != driver_id)
            .unwrap_or(false);

        if changed_driver {
            self.provider_conversation_id = None;
        }
        self.active_driver_id = Some(driver_id.to_string());
        if let Some(conv_id) = new_conversation_id.filter(|s| !s.trim().is_empty()) {
            self.provider_conversation_id = Some(conv_id);
        }
    }

    /// Inicia la Fase 1 (Reconocimiento) para un objetivo dado.
    pub fn start_recon(&mut self, objective: impl Into<String>) -> Result<(), HarnessTransitionError> {
        let obj = objective.into();
        if obj.trim().is_empty() {
            return Err(HarnessTransitionError("El objetivo no puede estar vacío".to_string()));
        }

        match &self.phase {
            HarnessPhase::Idle | HarnessPhase::Completed { .. } => {
                self.generation = self.generation.wrapping_add(1);
                self.turns.clear();
                self.provider_conversation_id = None;
                self.phase = HarnessPhase::Reconnaissance { objective: obj };
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "No se puede iniciar reconocimiento desde el estado actual: {:?}",
                other
            ))),
        }
    }

    /// Inicia o continúa la sesión conversacional SIN borrar la memoria descubierta del servidor.
    /// Solo `cancel_mission()` o `clear_conversation()` reinician el historial.
    pub fn continue_conversation(&mut self, objective: impl Into<String>) {
        let obj = objective.into();
        if matches!(self.phase, HarnessPhase::Idle | HarnessPhase::Completed { .. }) {
            self.phase = HarnessPhase::Reconnaissance { objective: obj };
        }
    }

    /// Marca la sesión como lista en reposo conservando `turns` y `provider_conversation_id`.
    pub fn conclude_turn_preserving_memory(&mut self) {
        if !matches!(self.phase, HarnessPhase::Idle | HarnessPhase::PausedOnDisconnect { .. }) {
            self.phase = HarnessPhase::Idle;
        }
    }

    /// Avanza de Fase 1 (Reconocimiento) a Fase 2 (Plan y Debate).
    pub fn advance_to_plan(
        &mut self,
        plan: Vec<AgentTaskProposal>,
    ) -> Result<(), HarnessTransitionError> {
        if plan.is_empty() {
            return Err(HarnessTransitionError(
                "No se puede pasar a PlanAndDebate con un plan vacío".to_string(),
            ));
        }
        for task in &plan {
            task.validate().map_err(HarnessTransitionError)?;
        }

        match &self.phase {
            HarnessPhase::Reconnaissance { objective } => {
                self.phase = HarnessPhase::PlanAndDebate {
                    objective: objective.clone(),
                    plan,
                };
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede avanzar a Plan y Debate desde Reconocimiento. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Actualiza el plan durante la Fase 2 tras un cuestionamiento o debate de ruta.
    pub fn update_plan_in_debate(
        &mut self,
        updated_plan: Vec<AgentTaskProposal>,
    ) -> Result<(), HarnessTransitionError> {
        if updated_plan.is_empty() {
            return Err(HarnessTransitionError(
                "El plan actualizado no puede quedar vacío".to_string(),
            ));
        }
        for task in &updated_plan {
            task.validate().map_err(HarnessTransitionError)?;
        }

        match &mut self.phase {
            HarnessPhase::PlanAndDebate { plan, .. } => {
                *plan = updated_plan;
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede actualizar el plan dentro de PlanAndDebate. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Aprueba el plan y activa la Fase 3 (Ejecución guiada) comenzando en la primera tarea.
    pub fn approve_plan_and_start_execution(&mut self) -> Result<(), HarnessTransitionError> {
        match &self.phase {
            HarnessPhase::PlanAndDebate { objective, plan } => {
                if plan.is_empty() {
                    return Err(HarnessTransitionError(
                        "No se puede iniciar ejecución con un plan vacío".to_string(),
                    ));
                }
                self.phase = HarnessPhase::GuidedExecution {
                    objective: objective.clone(),
                    tasks: plan.clone(),
                    current_step_index: 0,
                };
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede aprobar el plan desde PlanAndDebate. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Marca la tarea actual como completada y avanza a la siguiente o a Completed.
    pub fn complete_current_step(&mut self) -> Result<bool, HarnessTransitionError> {
        match &mut self.phase {
            HarnessPhase::GuidedExecution {
                objective,
                tasks,
                current_step_index,
            } => {
                if *current_step_index < tasks.len() {
                    tasks[*current_step_index].is_done = true;
                    *current_step_index += 1;
                }

                if *current_step_index >= tasks.len() {
                    let total = tasks.len();
                    let obj = objective.clone();
                    self.phase = HarnessPhase::Completed {
                        objective: obj,
                        completed_tasks_count: total,
                    };
                    Ok(true) // Misión finalizada
                } else {
                    Ok(false) // Quedan pasos pendientes
                }
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se pueden completar tareas durante GuidedExecution. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Frena la ejecución por error de comando remoto (`exit_code != 0`) conservando el plan.
    pub fn halt_on_error(
        &mut self,
        failed_command: impl Into<String>,
        exit_code: i32,
        stderr: impl Into<String>,
    ) -> Result<(), HarnessTransitionError> {
        match &self.phase {
            HarnessPhase::GuidedExecution {
                objective,
                tasks,
                current_step_index,
            } => {
                self.phase = HarnessPhase::HaltedOnError {
                    objective: objective.clone(),
                    tasks: tasks.clone(),
                    step_index: *current_step_index,
                    failed_command: failed_command.into(),
                    exit_code,
                    stderr: stderr.into(),
                };
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede frenar por error desde GuidedExecution. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Actualiza el comando (y su riesgo re-auditado) del paso actual durante `GuidedExecution`.
    pub fn update_current_step_command(
        &mut self,
        edited_command: impl Into<String>,
        audited_risk: RiskLevel,
    ) -> Result<(usize, AgentTaskProposal), HarnessTransitionError> {
        let cmd = edited_command.into();
        if cmd.trim().is_empty() {
            return Err(HarnessTransitionError(
                "El comando editado no puede estar vacío".to_string(),
            ));
        }

        match &mut self.phase {
            HarnessPhase::GuidedExecution {
                tasks,
                current_step_index,
                ..
            } => {
                let idx = *current_step_index;
                let Some(task) = tasks.get_mut(idx) else {
                    return Err(HarnessTransitionError(
                        "Índice de paso actual fuera de rango".to_string(),
                    ));
                };
                task.command = cmd.trim().to_string();
                task.risk_level = CommandRiskGuard::assess_and_elevate(audited_risk, &task.command);
                Ok((idx, task.clone()))
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede editar el paso activo durante GuidedExecution. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Reanuda la ejecución tras corregir el paso fallido, preservando el plan íntegro y re-auditando riesgo.
    pub fn resume_from_error(
        &mut self,
        fixed_command: Option<String>,
    ) -> Result<(), HarnessTransitionError> {
        match &self.phase {
            HarnessPhase::HaltedOnError {
                objective,
                tasks,
                step_index,
                ..
            } => {
                let idx = *step_index;
                let obj = objective.clone();
                let mut restored_tasks = tasks.clone();
                if let Some(cmd) = fixed_command.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()) {
                    if let Some(target_task) = restored_tasks.get_mut(idx) {
                        target_task.risk_level =
                            CommandRiskGuard::assess_and_elevate(target_task.risk_level, &cmd);
                        target_task.command = cmd;
                    }
                }
                self.phase = HarnessPhase::GuidedExecution {
                    objective: obj,
                    tasks: restored_tasks,
                    current_step_index: idx,
                };
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede reanudar desde HaltedOnError. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Reemplaza los pasos pendientes desde `step_index` en `HaltedOnError`, conservando `0..step_index`.
    pub fn replace_remaining_plan_from_error(
        &mut self,
        replacement_tasks: Vec<AgentTaskProposal>,
    ) -> Result<(), HarnessTransitionError> {
        if replacement_tasks.is_empty() {
            return Err(HarnessTransitionError(
                "El sub-plan de reemplazo no puede estar vacío".to_string(),
            ));
        }
        for task in &replacement_tasks {
            task.validate().map_err(HarnessTransitionError)?;
        }

        match &self.phase {
            HarnessPhase::HaltedOnError {
                objective,
                tasks,
                step_index,
                ..
            } => {
                let idx = (*step_index).min(tasks.len());
                let mut merged = Vec::with_capacity(idx + replacement_tasks.len());
                merged.extend_from_slice(&tasks[..idx]);
                for (offset, mut new_task) in replacement_tasks.into_iter().enumerate() {
                    new_task.id = (idx + offset + 1) as u32;
                    new_task.is_done = false;
                    new_task.is_rolled_back = false;
                    merged.push(new_task);
                }
                self.phase = HarnessPhase::GuidedExecution {
                    objective: objective.clone(),
                    tasks: merged,
                    current_step_index: idx,
                };
                Ok(())
            }
            other => Err(HarnessTransitionError(format!(
                "Solo se puede reemplazar la cola pendiente desde HaltedOnError. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Valida si `step_index` puede revertirse en el estado actual y devuelve el `rollback_cmd` efectivo.
    pub fn validate_rollback_request(
        &self,
        step_index: usize,
        custom_rollback_cmd: Option<&str>,
    ) -> Result<String, HarnessTransitionError> {
        let tasks = match &self.phase {
            HarnessPhase::GuidedExecution {
                tasks,
                current_step_index,
                ..
            } => {
                if step_index >= *current_step_index {
                    return Err(HarnessTransitionError(
                        "En ejecución guiada solo se pueden revertir pasos previos ya completados".to_string(),
                    ));
                }
                tasks
            }
            HarnessPhase::HaltedOnError {
                tasks,
                step_index: halted_idx,
                ..
            } => {
                if step_index > *halted_idx {
                    return Err(HarnessTransitionError(
                        "No se puede revertir un paso futuro que aún no ha sido ejecutado".to_string(),
                    ));
                }
                if step_index < *halted_idx && !tasks.get(step_index).map(|t| t.is_done).unwrap_or(false) {
                    return Err(HarnessTransitionError(
                        "El paso previo indicado no fue completado".to_string(),
                    ));
                }
                tasks
            }
            other => {
                return Err(HarnessTransitionError(format!(
                    "Rollback solo permitido en GuidedExecution o HaltedOnError. Estado actual: {:?}",
                    other
                )));
            }
        };

        let Some(task) = tasks.get(step_index) else {
            return Err(HarnessTransitionError(
                "Índice de tarea para rollback fuera de rango".to_string(),
            ));
        };
        if task.is_rolled_back {
            return Err(HarnessTransitionError(format!(
                "La tarea #{} ya fue revertida previamente",
                task.id
            )));
        }

        let effective = custom_rollback_cmd
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| task.rollback_cmd.trim());

        if effective.is_empty() {
            return Err(HarnessTransitionError(format!(
                "La tarea #{} no tiene comando de rollback definido",
                task.id
            )));
        }

        Ok(effective.to_string())
    }

    /// Registra de forma idempotente que `step_index` completó exitosamente su `rollback_cmd`.
    pub fn mark_step_rolled_back(&mut self, step_index: usize) -> Result<(), HarnessTransitionError> {
        let tasks = match &mut self.phase {
            HarnessPhase::GuidedExecution { tasks, .. }
            | HarnessPhase::HaltedOnError { tasks, .. } => tasks,
            other => {
                return Err(HarnessTransitionError(format!(
                    "No se puede marcar rollback en el estado actual: {:?}",
                    other
                )));
            }
        };

        let Some(task) = tasks.get_mut(step_index) else {
            return Err(HarnessTransitionError(
                "Índice de tarea fuera de rango al marcar rollback".to_string(),
            ));
        };
        if task.is_rolled_back {
            return Err(HarnessTransitionError(format!(
                "La tarea #{} ya estaba marcada como revertida",
                task.id
            )));
        }
        task.is_rolled_back = true;
        Ok(())
    }

    /// Pausa la máquina de estados conservando el snapshot activo ante desconexión SSH.
    pub fn pause_on_disconnect(&mut self) -> Result<(), HarnessTransitionError> {
        match &self.phase {
            HarnessPhase::Idle | HarnessPhase::Completed { .. } => Ok(()),
            HarnessPhase::PausedOnDisconnect { .. } => Ok(()),
            active => {
                self.phase = HarnessPhase::PausedOnDisconnect {
                    previous_phase: Box::new(active.clone()),
                };
                Ok(())
            }
        }
    }

    /// Reanuda la fase previa tras reconectar exitosamente la terminal.
    pub fn resume_after_reconnect(&mut self) -> Result<HarnessPhase, HarnessTransitionError> {
        match &mut self.phase {
            HarnessPhase::PausedOnDisconnect { previous_phase } => {
                let restored = *previous_phase.clone();
                self.phase = restored.clone();
                Ok(restored)
            }
            other => Err(HarnessTransitionError(format!(
                "El arnés no se encuentra en estado pausado por desconexión. Estado actual: {:?}",
                other
            ))),
        }
    }

    /// Cancela cualquier misión en curso, limpia sesión conversacional e incrementa `generation`.
    pub fn cancel_mission(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.phase = HarnessPhase::Idle;
        self.turns.clear();
        self.provider_conversation_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::schema::RiskLevel;

    fn sample_plan() -> Vec<AgentTaskProposal> {
        vec![
            AgentTaskProposal::new(
                1,
                "Instalar paquetes",
                "Tomcat requiere Java",
                "apt-get install -y default-jdk",
                RiskLevel::ConfigChange,
                "dpkg --get-selections",
                "apt-get purge -y default-jdk",
            ),
            AgentTaskProposal::new(
                2,
                "Iniciar servicio",
                "Iniciar Tomcat",
                "systemctl start tomcat",
                RiskLevel::ServiceImpact,
                "systemctl status tomcat",
                "systemctl stop tomcat",
            ),
        ]
    }

    #[test]
    fn flujo_completo_feliz_de_tres_fases() {
        let mut fsm = AgentHarnessFsm::new("term-1", ExecutionMode::Strict);
        assert_eq!(*fsm.phase(), HarnessPhase::Idle);

        fsm.start_recon("Instalar Tomcat").expect("start recon");
        assert!(matches!(fsm.phase(), HarnessPhase::Reconnaissance { .. }));

        fsm.advance_to_plan(sample_plan()).expect("advance plan");
        assert!(matches!(fsm.phase(), HarnessPhase::PlanAndDebate { .. }));

        fsm.approve_plan_and_start_execution().expect("approve");
        assert!(matches!(
            fsm.phase(),
            HarnessPhase::GuidedExecution {
                current_step_index: 0,
                ..
            }
        ));

        let done1 = fsm.complete_current_step().expect("step 1");
        assert!(!done1);
        assert!(matches!(
            fsm.phase(),
            HarnessPhase::GuidedExecution {
                current_step_index: 1,
                ..
            }
        ));

        let done2 = fsm.complete_current_step().expect("step 2");
        assert!(done2);
        assert!(matches!(
            fsm.phase(),
            HarnessPhase::Completed {
                completed_tasks_count: 2,
                ..
            }
        ));
    }

    #[test]
    fn freno_automatico_preserva_plan_y_aplica_comando_corregido() {
        let mut fsm = AgentHarnessFsm::new("term-1", ExecutionMode::Strict);
        fsm.start_recon("Despliegue").unwrap();
        fsm.advance_to_plan(sample_plan()).unwrap();
        fsm.approve_plan_and_start_execution().unwrap();

        fsm.halt_on_error("apt-get install -y default-jdk", 100, "Package not found")
            .unwrap();
        match fsm.phase() {
            HarnessPhase::HaltedOnError {
                tasks,
                step_index,
                exit_code,
                ..
            } => {
                assert_eq!(*step_index, 0);
                assert_eq!(*exit_code, 100);
                assert_eq!(tasks.len(), 2);
            }
            other => panic!("Estado inesperado: {:?}", other),
        }

        // Reanudar con comando corregido debe mantener ambas tareas y actualizar la tarea 0
        fsm.resume_from_error(Some("apt-get update && apt-get install -y openjdk-21-jre".to_string()))
            .unwrap();
        match fsm.phase() {
            HarnessPhase::GuidedExecution {
                tasks,
                current_step_index,
                ..
            } => {
                assert_eq!(*current_step_index, 0);
                assert_eq!(tasks.len(), 2);
                assert_eq!(
                    tasks[0].command,
                    "apt-get update && apt-get install -y openjdk-21-jre"
                );
                assert_eq!(tasks[1].title, "Iniciar servicio");
            }
            other => panic!("Estado inesperado al reanudar: {:?}", other),
        }
    }

    #[test]
    fn congelamiento_por_desconexion_y_reanudacion() {
        let mut fsm = AgentHarnessFsm::new("term-1", ExecutionMode::Flexible);
        fsm.start_recon("Prueba de red").unwrap();
        fsm.advance_to_plan(sample_plan()).unwrap();
        fsm.approve_plan_and_start_execution().unwrap();

        fsm.pause_on_disconnect().unwrap();
        assert!(matches!(fsm.phase(), HarnessPhase::PausedOnDisconnect { .. }));

        let restored = fsm.resume_after_reconnect().unwrap();
        assert!(matches!(restored, HarnessPhase::GuidedExecution { .. }));
        assert_eq!(*fsm.phase(), restored);
    }

    #[test]
    fn cambio_de_driver_invalida_conversation_id_pero_conserva_turns_portatiles() {
        let mut fsm = AgentHarnessFsm::new("term-1", ExecutionMode::Strict);
        fsm.start_recon("Optimizar nginx").unwrap();
        fsm.record_turn(ConversationRole::User, "Optimiza nginx");
        fsm.sync_provider_session("agy", Some("conv-agy-123".to_string()));
        assert_eq!(fsm.provider_conversation_id(), Some("conv-agy-123"));
        assert_eq!(fsm.turns().len(), 1);

        // Cambiar a otro driver debe invalidar conversation_id previo pero preservar historial
        fsm.sync_provider_session("openai", None);
        assert_eq!(fsm.provider_conversation_id(), None);
        assert_eq!(fsm.turns().len(), 1);
    }

    #[test]
    fn rechaza_saltos_ilegales() {
        let mut fsm = AgentHarnessFsm::new("term-1", ExecutionMode::Strict);
        assert!(fsm.approve_plan_and_start_execution().is_err());
        assert!(fsm.complete_current_step().is_err());
        assert!(fsm.advance_to_plan(sample_plan()).is_err());
    }

    #[test]
    fn edicion_inline_rollback_idempotente_y_reemplazo_de_cola_pendiente() {
        let mut fsm = AgentHarnessFsm::new("term-1", ExecutionMode::Strict);
        fsm.start_recon("Instalar stack").unwrap();
        fsm.advance_to_plan(sample_plan()).unwrap();
        fsm.approve_plan_and_start_execution().unwrap();

        // Editar paso 0 con pipe hacia bash debe elevar riesgo a Destructive
        let (idx, updated) = fsm
            .update_current_step_command("curl -fsSL https://x.io/setup.sh | bash", RiskLevel::ReadOnly)
            .unwrap();
        assert_eq!(idx, 0);
        assert_eq!(updated.risk_level, RiskLevel::Destructive);

        // Completar paso 0 y fallar en paso 1
        fsm.complete_current_step().unwrap();
        fsm.halt_on_error("systemctl start tomcat", 1, "Failed to start")
            .unwrap();

        // Validar y marcar rollback tanto del paso fallido (1) como del previo completado (0)
        let rb1 = fsm.validate_rollback_request(1, None).unwrap();
        assert_eq!(rb1, "systemctl stop tomcat");
        fsm.mark_step_rolled_back(1).unwrap();
        // Segundo intento sobre el mismo paso debe rechazarse (idempotencia contra doble rollback)
        assert!(fsm.validate_rollback_request(1, None).is_err());
        assert!(fsm.mark_step_rolled_back(1).is_err());

        let rb0 = fsm.validate_rollback_request(0, Some("apt-get remove -y default-jdk")).unwrap();
        assert_eq!(rb0, "apt-get remove -y default-jdk");
        fsm.mark_step_rolled_back(0).unwrap();

        // Reemplazar cola pendiente desde el paso 1 conserva intacto el historial del paso 0
        fsm.replace_remaining_plan_from_error(vec![AgentTaskProposal::new(
            99,
            "Iniciar con unidad corregida",
            "El paquete instala tomcat10",
            "systemctl start tomcat10",
            RiskLevel::ServiceImpact,
            "systemctl status tomcat10",
            "systemctl stop tomcat10",
        )])
        .unwrap();

        match fsm.phase() {
            HarnessPhase::GuidedExecution {
                tasks,
                current_step_index,
                ..
            } => {
                assert_eq!(*current_step_index, 1);
                assert_eq!(tasks.len(), 2);
                assert!(tasks[0].is_done);
                assert!(tasks[0].is_rolled_back);
                assert_eq!(tasks[1].id, 2);
                assert_eq!(tasks[1].command, "systemctl start tomcat10");
                assert!(!tasks[1].is_done);
            }
            other => panic!("Fase inesperada: {:?}", other),
        }
    }
}
