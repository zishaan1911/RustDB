// Parser nodes - predict.rs placeholder
//! PREDICT statement AST node

pub struct PredictNode {
    pub model_name: String,
    pub input_data: String,
    pub features: Vec<String>,
}
