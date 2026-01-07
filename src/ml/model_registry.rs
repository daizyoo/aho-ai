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
    /// Auto-discover models in the models directory (recursive)
    pub fn discover_models<P: AsRef<Path>>(&mut self, models_dir: P) -> anyhow::Result<()> {
        let models_dir = models_dir.as_ref();

        if !models_dir.exists() {
            return Ok(());
        }

        // Use a stack for recursive traversal
        let mut dirs_to_visit = vec![models_dir.to_path_buf()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            if let Ok(entries) = std::fs::read_dir(&current_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();

                        if path.is_dir() {
                            dirs_to_visit.push(path);
                            continue;
                        }

                        let extension = path.extension().and_then(|s| s.to_str());
                        let model_type = match extension {
                            Some("onnx") => Some(ModelType::ONNX),
                            _ => None,
                        };

                        if let Some(model_type) = model_type {
                            // Calculate relative name from base directory
                            let relative_path = path.strip_prefix(models_dir).unwrap_or(&path);
                            let mut name = relative_path
                                .with_extension("") // Remove .onnx
                                .to_string_lossy()
                                .into_owned();

                            // Strip "model" from end if it exists (e.g. Fair/v1.0/model -> Fair/v1.0)
                            if name.ends_with("/model") {
                                name = name.strip_suffix("/model").unwrap().to_string();
                            } else if name == "model" {
                                // Keep it if it's just "model" at the root, or handle as needed
                            }

                            let version = {
                                #[cfg(feature = "ml")]
                                {
                                    // Try to load session to read metadata
                                    if let Ok(session) =
                                        Session::builder().and_then(|b| b.commit_from_file(&path))
                                    {
                                        if let Ok(metadata) = session.metadata() {
                                            if let Ok(Some(version)) = metadata.custom("version") {
                                                Some(version)
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                                #[cfg(not(feature = "ml"))]
                                {
                                    None
                                }
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
            }
        }

        Ok(())
    }

    /// Helper to read version from an ONNX file
    pub fn get_model_version(path: &Path) -> Option<String> {
        #[cfg(feature = "ml")]
        {
            if let Ok(session) = Session::builder().and_then(|b| b.commit_from_file(path)) {
                if let Ok(metadata) = session.metadata() {
                    if let Ok(Some(version)) = metadata.custom("version") {
                        return Some(version);
                    }
                }
            }
        }
        None
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
