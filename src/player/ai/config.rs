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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub max_depth_light: u8,
    pub max_depth_strong: u8,
}

impl AIConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = "ai_config.json";
        let config_str = std::fs::read_to_string(config_path)?;
        let config: AIConfig = serde_json::from_str(&config_str)?;
        Ok(config)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_else(|_| Self::default())
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
                hand_piece_bonus_multiplier: 1.2,
                material_values,
                pst_enabled: true,
            },
            search: SearchConfig {
                max_depth_light: 2,
                max_depth_strong: 4,
            },
        }
    }
}
