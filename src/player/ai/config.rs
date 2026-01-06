use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub version: String,
    pub evaluation: EvaluationConfig,
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub hand_piece_bonus_multiplier: f64,
    pub material_values: HashMap<String, i32>,
    pub pst_enabled: bool,
    #[serde(default = "default_evaluator_type")]
    pub evaluator_type: String,
    #[serde(default)]
    pub nn_model_path: Option<String>,
}

fn default_evaluator_type() -> String {
    "Handcrafted".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub max_depth_light: u8,
    pub max_depth_strong: u8,
}

// Global config instance - loaded once at startup
pub static AI_CONFIG: Lazy<AIConfig> = Lazy::new(|| {
    AIConfig::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load ai_config.json: {}", e);
        eprintln!("Using default configuration");
        AIConfig::default()
    })
});

impl AIConfig {
    fn load() -> anyhow::Result<Self> {
        let config_path = "ai_config.json";
        let config_str = std::fs::read_to_string(config_path)?;
        let config: AIConfig = serde_json::from_str(&config_str)?;
        Ok(config)
    }

    /// Get the global config instance (zero-cost after first access)
    pub fn get() -> &'static AIConfig {
        &AI_CONFIG
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        let mut material_values = HashMap::new();
        material_values.insert("pawn".to_string(), 100);
        material_values.insert("lance".to_string(), 350);
        material_values.insert("knight".to_string(), 400);
        material_values.insert("silver".to_string(), 550);
        material_values.insert("gold".to_string(), 600);
        material_values.insert("bishop".to_string(), 850);
        material_values.insert("rook".to_string(), 1000);
        material_values.insert("king".to_string(), 10000);

        AIConfig {
            version: "1.0".to_string(),
            evaluation: EvaluationConfig {
                hand_piece_bonus_multiplier: 1.1,
                material_values,
                pst_enabled: true,
                evaluator_type: "Handcrafted".to_string(),
                nn_model_path: None,
            },
            search: SearchConfig {
                max_depth_light: 2,
                max_depth_strong: 4,
            },
        }
    }
}
