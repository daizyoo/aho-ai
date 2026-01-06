use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "ml")]
use ort::session::Session;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelType {
    ONNX,
    // Future: PyTorch, TensorFlow, etc.
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub name: String,
    pub model_type: ModelType,
    pub path: PathBuf,
    pub version: Option<String>,
    pub created_at: Option<String>,
}

/// Registry for managing available ML models
pub struct ModelRegistry {
    models: HashMap<String, ModelMetadata>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Register a new model in the registry
    pub fn register(&mut self, metadata: ModelMetadata) {
        self.models.insert(metadata.name.clone(), metadata);
    }

    /// Get model metadata by name
    pub fn get(&self, name: &str) -> Option<&ModelMetadata> {
        self.models.get(name)
    }

    /// List all registered models
    pub fn list(&self) -> Vec<&ModelMetadata> {
        self.models.values().collect()
    }

    /// Auto-discover models in the models directory
    pub fn discover_models<P: AsRef<Path>>(&mut self, models_dir: P) -> anyhow::Result<()> {
        let models_dir = models_dir.as_ref();

        if !models_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(models_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let extension = path.extension().and_then(|s| s.to_str());

                let model_type = match extension {
                    Some("onnx") => Some(ModelType::ONNX),
                    _ => None,
                };

                if let Some(model_type) = model_type {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let version = {
                        let mut v = None;
                        #[cfg(feature = "ml")]
                        {
                            // Try to load session to read metadata
                            if let Ok(session) =
                                Session::builder().and_then(|b| b.commit_from_file(&path))
                            {
                                if let Ok(metadata) = session.metadata() {
                                    if let Ok(Some(version)) = metadata.custom("version") {
                                        v = Some(version);
                                    }
                                }
                            }
                        }
                        v
                    };

                    let metadata = ModelMetadata {
                        name: name.clone(),
                        model_type,
                        path: path.clone(),
                        version,
                        created_at: None,
                    };

                    self.register(metadata);
                }
            }
        }

        Ok(())
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let mut registry = ModelRegistry::new();

        let metadata = ModelMetadata {
            name: "test_model".to_string(),
            model_type: ModelType::ONNX,
            path: PathBuf::from("models/test.onnx"),
            version: Some("1.0".to_string()),
            created_at: None,
        };

        registry.register(metadata.clone());

        assert_eq!(
            registry.get("test_model").map(|m| &m.name),
            Some(&"test_model".to_string())
        );
        assert_eq!(registry.list().len(), 1);
    }
}
