// Parser nodes - train_model.rs placeholder
//! TRAIN MODEL statement AST node

pub struct TrainModelNode {
    pub model_name: String,
    pub algorithm: Algorithm,
    pub training_data: String,
    pub features: Vec<String>,
    pub label: String,
    pub hyperparameters: std::collections::HashMap<String, String>,
}

pub enum Algorithm {
    LinearRegression,
    LogisticRegression,
    KMeans,
    DecisionTree,
}
