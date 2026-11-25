//! Driver simulado determinista (`FakeAiDriver`) para pruebas unitarias e integración sin red.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::ai::driver::{
    AgentTurnOutcome, AgentTurnRequest, AiDriverError, AiModelDescriptor, AiProviderDriver,
    DriverHealthStatus, TurnCancellationToken,
};

#[derive(Clone)]
pub struct FakeAiDriver {
    driver_id: String,
    health: DriverHealthStatus,
    models: Vec<AiModelDescriptor>,
    queued_responses: Arc<Mutex<VecDeque<Result<AgentTurnOutcome, AiDriverError>>>>,
    recorded_requests: Arc<Mutex<Vec<AgentTurnRequest>>>,
    progress_chunks: Vec<String>,
    simulated_delay_ms: u64,
}

impl FakeAiDriver {
    pub fn new() -> Self {
        Self {
            driver_id: "fake".to_string(),
            health: DriverHealthStatus {
                driver_id: "fake".to_string(),
                available: true,
                version: Some("1.0.0-test".to_string()),
                message: "Driver de prueba listo".to_string(),
            },
            models: vec![
                AiModelDescriptor {
                    id: "fake-fast".to_string(),
                    label: "Fake Fast".to_string(),
                },
                AiModelDescriptor {
                    id: "fake-pro".to_string(),
                    label: "Fake Pro".to_string(),
                },
            ],
            queued_responses: Arc::new(Mutex::new(VecDeque::new())),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
            progress_chunks: vec!["Analizando contexto...".to_string()],
            simulated_delay_ms: 0,
        }
    }

    pub fn with_driver_id(mut self, id: impl Into<String>) -> Self {
        let id = id.into();
        self.health.driver_id = id.clone();
        self.driver_id = id;
        self
    }

    pub fn with_health(mut self, health: DriverHealthStatus) -> Self {
        self.health = health;
        self
    }

    pub fn with_models(mut self, models: Vec<AiModelDescriptor>) -> Self {
        self.models = models;
        self
    }

    pub fn with_progress_chunks(mut self, chunks: Vec<String>) -> Self {
        self.progress_chunks = chunks;
        self
    }

    pub fn with_delay_ms(mut self, delay_ms: u64) -> Self {
        self.simulated_delay_ms = delay_ms;
        self
    }

    pub fn push_response(&self, outcome: Result<AgentTurnOutcome, AiDriverError>) {
        self.queued_responses.lock().unwrap().push_back(outcome);
    }

    pub fn recorded_requests(&self) -> Vec<AgentTurnRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }
}

impl Default for FakeAiDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl AiProviderDriver for FakeAiDriver {
    fn driver_id(&self) -> &str {
        &self.driver_id
    }

    fn probe_health(&self) -> DriverHealthStatus {
        self.health.clone()
    }

    fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiDriverError> {
        if !self.health.available {
            return Err(AiDriverError::BinaryNotFound(self.health.message.clone()));
        }
        Ok(self.models.clone())
    }

    fn execute_turn(
        &self,
        request: &AgentTurnRequest,
        cancel: &TurnCancellationToken,
        on_progress: &(dyn Fn(&str) + Send + Sync),
    ) -> Result<AgentTurnOutcome, AiDriverError> {
        self.recorded_requests.lock().unwrap().push(request.clone());

        for chunk in &self.progress_chunks {
            if cancel.is_cancelled() {
                return Err(AiDriverError::Cancelled);
            }
            on_progress(chunk);
        }

        if self.simulated_delay_ms > 0 {
            let steps = (self.simulated_delay_ms / 10).max(1);
            for _ in 0..steps {
                if cancel.is_cancelled() {
                    return Err(AiDriverError::Cancelled);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        if cancel.is_cancelled() {
            return Err(AiDriverError::Cancelled);
        }

        self.queued_responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| {
                Err(AiDriverError::InvalidResponse(
                    "No hay respuestas programadas en FakeAiDriver".to_string(),
                ))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::schema::AgentResponseKind;
    use std::sync::Mutex as StdMutex;

    #[test]
    fn fake_driver_emite_progreso_y_devuelve_turno() {
        let driver = FakeAiDriver::new()
            .with_progress_chunks(vec!["Pensando...".to_string(), "Validando...".to_string()]);
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ReplyOnly {
                message: "Listo".to_string(),
                phase_tag: None,
            },
            provider_conversation_id: Some("conv-1".to_string()),
            raw_model_used: Some("fake-fast".to_string()),
        }));

        let req = AgentTurnRequest {
            system_prompt: "sys".to_string(),
            user_prompt: "hola".to_string(),
            terminal_context: None,
            turns: Vec::new(),
            provider_conversation_id: None,
            selected_model: Some("fake-fast".to_string()),
            json_schema: serde_json::json!({}),
            timeout_secs: 30,
        };

        let captured = StdMutex::new(Vec::new());
        let cancel = TurnCancellationToken::new();
        let out = driver
            .execute_turn(&req, &cancel, &|chunk| {
                captured.lock().unwrap().push(chunk.to_string());
            })
            .expect("ok");

        assert_eq!(captured.lock().unwrap().len(), 2);
        assert_eq!(out.provider_conversation_id.as_deref(), Some("conv-1"));
        assert_eq!(driver.recorded_requests().len(), 1);
    }

    #[test]
    fn fake_driver_respeta_cancelacion() {
        let driver = FakeAiDriver::new().with_delay_ms(200);
        driver.push_response(Ok(AgentTurnOutcome {
            response: AgentResponseKind::ReplyOnly {
                message: "No debería llegar".to_string(),
                phase_tag: None,
            },
            provider_conversation_id: None,
            raw_model_used: None,
        }));

        let req = AgentTurnRequest {
            system_prompt: "sys".to_string(),
            user_prompt: "hola".to_string(),
            terminal_context: None,
            turns: Vec::new(),
            provider_conversation_id: None,
            selected_model: None,
            json_schema: serde_json::json!({}),
            timeout_secs: 30,
        };

        let cancel = TurnCancellationToken::new();
        cancel.cancel();
        let err = driver.execute_turn(&req, &cancel, &|_| {}).unwrap_err();
        assert_eq!(err, AiDriverError::Cancelled);
    }
}
