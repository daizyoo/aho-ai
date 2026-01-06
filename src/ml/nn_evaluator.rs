//! Neural Network Evaluator using ONNX Runtime

#[cfg(feature = "ml")]
use ndarray::Array1;
#[cfg(feature = "ml")]
use ort::{inputs, session::Session, value::Value};
#[cfg(feature = "ml")]
use std::sync::Mutex;

use crate::core::{Board, PlayerId};
use crate::ml::features::BoardFeatureExtractor;
use crate::player::ai::evaluator::Evaluator;

#[cfg(feature = "ml")]
pub struct NNEvaluator {
    session: Mutex<Session>,
}

#[cfg(feature = "ml")]
impl NNEvaluator {
    /// Load ONNX model from file
    pub fn load(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let session = Session::builder()?.commit_from_file(model_path)?;

        // Try to read version metadata
        if let Ok(metadata) = session.metadata() {
            if let Ok(Some(version)) = metadata.custom("version") {
                eprintln!("[ML] Loaded model: {} (v{})\r", model_path, version);
            } else {
                eprintln!("[ML] Loaded model: {} (no version)\r", model_path);
            }
        }

        Ok(Self {
            session: Mutex::new(session),
        })
    }

    /// Run inference on board state
    fn run_inference(&self, features: &[f32]) -> Result<f32, Box<dyn std::error::Error>> {
        // Create input array with batch dimension [1, features_size]
        let input_array: Array1<f32> = Array1::from_vec(features.to_vec());
        let input_array = input_array.insert_axis(ndarray::Axis(0));

        // Create Value from array
        let input_value = Value::from_array(input_array)?;

        // Run inference (lock mutex to get mutable access)
        let mut session = self.session.lock().unwrap();
        let outputs = session.run(inputs!["board_features" => input_value])?;

        // Extract value from output (second output is value head)
        let value_tensor = outputs.get("value").ok_or("Could not find value output")?;

        // Extract tensor data
        let (_shape, data) = value_tensor.try_extract_tensor::<f32>()?;
        let value = data.first().copied().unwrap_or(0.0);

        Ok(value)
    }
}

#[cfg(feature = "ml")]
impl Evaluator for NNEvaluator {
    fn evaluate(&mut self, board: &Board) -> i32 {
        let features = BoardFeatureExtractor::extract(board, PlayerId::Player1);

        // Use static counter to show occasional confirmation
        static EVAL_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let count = EVAL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        match self.run_inference(&features) {
            Ok(value) => {
                let score = (value * 10000.0) as i32;
                // Show confirmation every 100 evaluations
                if count % 100 == 0 {
                    // eprintln!(
                    //     "[ML] Evaluation #{}: NN value={:.4} → score={}\r",
                    //     count, value, score
                    // );
                }
                score
            }
            Err(e) => {
                eprintln!("[ERROR] NN inference failed: {}\r", e);
                0
            }
        }
    }

    fn name(&self) -> &str {
        "NeuralNetwork"
    }
}

// Stub when ml feature disabled
#[cfg(not(feature = "ml"))]
pub struct NNEvaluator;

#[cfg(not(feature = "ml"))]
impl NNEvaluator {
    pub fn load(_model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Err("ML feature not enabled. Rebuild with --features ml".into())
    }
}

#[cfg(not(feature = "ml"))]
impl Evaluator for NNEvaluator {
    fn evaluate(&mut self, _board: &Board) -> i32 {
        0
    }

    fn name(&self) -> &str {
        "NeuralNetwork (disabled)"
    }
}
