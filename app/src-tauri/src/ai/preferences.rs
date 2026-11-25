//! Persistencia y consulta de directivas universales de IA en SQLite (`app_preferences`).

use std::collections::HashMap;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use crate::preferences::{get_preference, set_preference};
use super::fsm::ExecutionMode;
use super::prompt::{DEFAULT_OPERATOR_INFRA_RULES, DEFAULT_OPERATOR_PERSONA};

pub const AI_EXECUTION_MODE_KEY: &str = "ai_execution_mode";
pub const AI_OPERATOR_PERSONA_KEY: &str = "ai_operator_persona_prompt";
pub const AI_OPERATOR_INFRA_RULES_KEY: &str = "ai_operator_infra_rules";
pub const AI_ACTIVE_DRIVER_KEY: &str = "ai_active_driver";
pub const AI_SELECTED_MODEL_KEY: &str = "ai_selected_model";
pub const AI_DRIVER_OPTIONS_JSON_KEY: &str = "ai_driver_options_json";
pub const AI_CONTEXT_MAX_LINES_KEY: &str = "ai_context_max_lines";

pub const DEFAULT_ACTIVE_DRIVER: &str = "agy";
pub const DEFAULT_CONTEXT_MAX_LINES: usize = 200;
pub const MIN_CONTEXT_MAX_LINES: usize = 50;
pub const MAX_CONTEXT_MAX_LINES: usize = 1000;

fn default_active_driver() -> String {
    DEFAULT_ACTIVE_DRIVER.to_string()
}

fn default_context_max_lines() -> usize {
    DEFAULT_CONTEXT_MAX_LINES
}

pub fn clamp_context_max_lines(lines: usize) -> usize {
    lines.clamp(MIN_CONTEXT_MAX_LINES, MAX_CONTEXT_MAX_LINES)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPreferencesDto {
    pub execution_mode: ExecutionMode,
    pub operator_persona: String,
    pub operator_infra_rules: String,
    #[serde(default = "default_active_driver")]
    pub active_driver: String,
    #[serde(default)]
    pub selected_model: String,
    #[serde(default = "default_context_max_lines")]
    pub context_max_lines: usize,
    #[serde(default)]
    pub driver_options: HashMap<String, String>,
}

impl Default for AiPreferencesDto {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Strict,
            operator_persona: DEFAULT_OPERATOR_PERSONA.to_string(),
            operator_infra_rules: DEFAULT_OPERATOR_INFRA_RULES.to_string(),
            active_driver: DEFAULT_ACTIVE_DRIVER.to_string(),
            selected_model: String::new(),
            context_max_lines: DEFAULT_CONTEXT_MAX_LINES,
            driver_options: HashMap::new(),
        }
    }
}

pub fn get_ai_preferences(conn: &Connection) -> Result<AiPreferencesDto, String> {
    let mode_str = get_preference(conn, AI_EXECUTION_MODE_KEY)?;
    let mode = if mode_str.trim().is_empty() {
        ExecutionMode::Strict
    } else {
        ExecutionMode::from_str_lenient(&mode_str)
    };

    let persona = get_preference(conn, AI_OPERATOR_PERSONA_KEY)?;
    let final_persona = if persona.trim().is_empty() {
        DEFAULT_OPERATOR_PERSONA.to_string()
    } else {
        persona
    };

    let rules = get_preference(conn, AI_OPERATOR_INFRA_RULES_KEY)?;
    let final_rules = if rules.trim().is_empty() {
        DEFAULT_OPERATOR_INFRA_RULES.to_string()
    } else {
        rules
    };

    let driver = get_preference(conn, AI_ACTIVE_DRIVER_KEY)?;
    let final_driver = if driver.trim().is_empty() {
        DEFAULT_ACTIVE_DRIVER.to_string()
    } else {
        driver.trim().to_string()
    };

    let selected_model = get_preference(conn, AI_SELECTED_MODEL_KEY)?.trim().to_string();

    let max_lines_str = get_preference(conn, AI_CONTEXT_MAX_LINES_KEY)?;
    let context_max_lines = max_lines_str
        .trim()
        .parse::<usize>()
        .map(clamp_context_max_lines)
        .unwrap_or(DEFAULT_CONTEXT_MAX_LINES);

    let options_json = get_preference(conn, AI_DRIVER_OPTIONS_JSON_KEY)?;
    let driver_options: HashMap<String, String> = if options_json.trim().is_empty() {
        HashMap::new()
    } else {
        serde_json::from_str(&options_json).unwrap_or_default()
    };

    Ok(AiPreferencesDto {
        execution_mode: mode,
        operator_persona: final_persona,
        operator_infra_rules: final_rules,
        active_driver: final_driver,
        selected_model,
        context_max_lines,
        driver_options,
    })
}

pub fn set_ai_preferences(conn: &Connection, prefs: &AiPreferencesDto) -> Result<(), String> {
    set_preference(conn, AI_EXECUTION_MODE_KEY, prefs.execution_mode.as_str())?;
    set_preference(conn, AI_OPERATOR_PERSONA_KEY, prefs.operator_persona.trim())?;
    set_preference(conn, AI_OPERATOR_INFRA_RULES_KEY, prefs.operator_infra_rules.trim())?;

    let driver = if prefs.active_driver.trim().is_empty() {
        DEFAULT_ACTIVE_DRIVER
    } else {
        prefs.active_driver.trim()
    };
    set_preference(conn, AI_ACTIVE_DRIVER_KEY, driver)?;
    set_preference(conn, AI_SELECTED_MODEL_KEY, prefs.selected_model.trim())?;

    let clamped_lines = clamp_context_max_lines(prefs.context_max_lines);
    set_preference(conn, AI_CONTEXT_MAX_LINES_KEY, &clamped_lines.to_string())?;

    let options_json = serde_json::to_string(&prefs.driver_options)
        .map_err(|e| format!("Error al serializar driver_options: {}", e))?;
    set_preference(conn, AI_DRIVER_OPTIONS_JSON_KEY, &options_json)?;
    Ok(())
}

pub fn reset_ai_preferences(conn: &Connection) -> Result<AiPreferencesDto, String> {
    let defaults = AiPreferencesDto::default();
    set_ai_preferences(conn, &defaults)?;
    Ok(defaults)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::ensure_app_preferences_schema;

    fn open_test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("mem");
        ensure_app_preferences_schema(&conn).expect("schema");
        conn
    }

    #[test]
    fn get_retorna_defaults_en_bd_vacia() {
        let conn = open_test_conn();
        let prefs = get_ai_preferences(&conn).expect("get");
        assert_eq!(prefs.execution_mode, ExecutionMode::Strict);
        assert_eq!(prefs.operator_persona, DEFAULT_OPERATOR_PERSONA);
        assert_eq!(prefs.operator_infra_rules, DEFAULT_OPERATOR_INFRA_RULES);
        assert_eq!(prefs.active_driver, "agy");
        assert_eq!(prefs.selected_model, "");
        assert_eq!(prefs.context_max_lines, 200);
        assert!(prefs.driver_options.is_empty());
    }

    #[test]
    fn set_y_get_roundtrip_incluyendo_driver_modelo_y_context_lines() {
        let conn = open_test_conn();
        let mut opts = HashMap::new();
        opts.insert("agy_binary_path".to_string(), r"C:\bin\agy.EXE".to_string());

        let custom = AiPreferencesDto {
            execution_mode: ExecutionMode::Flexible,
            operator_persona: "Custom persona".to_string(),
            operator_infra_rules: "Custom rules".to_string(),
            active_driver: "agy".to_string(),
            selected_model: "gemini-4-carbon-high".to_string(),
            context_max_lines: 350,
            driver_options: opts,
        };
        set_ai_preferences(&conn, &custom).expect("set");
        let fetched = get_ai_preferences(&conn).expect("get");
        assert_eq!(fetched, custom);
    }

    #[test]
    fn context_max_lines_se_acota_al_rango_seguro() {
        let conn = open_test_conn();
        let mut custom = AiPreferencesDto {
            context_max_lines: 5,
            ..Default::default()
        };
        set_ai_preferences(&conn, &custom).expect("set min");
        assert_eq!(get_ai_preferences(&conn).unwrap().context_max_lines, 50);

        custom.context_max_lines = 99999;
        set_ai_preferences(&conn, &custom).expect("set max");
        assert_eq!(get_ai_preferences(&conn).unwrap().context_max_lines, 1000);
    }

    #[test]
    fn reset_restaura_defaults() {
        let conn = open_test_conn();
        let mut opts = HashMap::new();
        opts.insert("agy_binary_path".to_string(), "/usr/local/bin/agy".to_string());
        let custom = AiPreferencesDto {
            execution_mode: ExecutionMode::Flexible,
            operator_persona: "Custom persona".to_string(),
            operator_infra_rules: "Custom rules".to_string(),
            active_driver: "openai".to_string(),
            selected_model: "gpt-5".to_string(),
            context_max_lines: 600,
            driver_options: opts,
        };
        set_ai_preferences(&conn, &custom).expect("set");
        let restored = reset_ai_preferences(&conn).expect("reset");
        assert_eq!(restored.execution_mode, ExecutionMode::Strict);
        assert_eq!(restored.active_driver, "agy");
        assert_eq!(restored.selected_model, "");
        assert_eq!(restored.context_max_lines, 200);
        assert!(restored.driver_options.is_empty());
        assert_eq!(restored.operator_persona, DEFAULT_OPERATOR_PERSONA);
    }
}
