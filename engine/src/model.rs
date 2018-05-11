use serde_json::from_str;
use features::{NUM_OUTCOMES, NUM_MODEL_FEATURES};
use policy_features::NUM_POLICY_FEATURES;

const DEFAULT_VALUE: &'static str = include_str!("../model");
const DEFAULT_POLICY: &'static str = include_str!("../policy_model");

#[derive(Serialize, Deserialize, Clone)]
pub struct Model {
    pub value_coef: Vec<[f32; NUM_OUTCOMES]>,
    pub policy_coef: Vec<f32>,
}

impl Model {
    pub fn zero() -> Self {
        let value_coef = vec![[0.0; NUM_OUTCOMES]; NUM_MODEL_FEATURES];
        let policy_coef = vec![0.0; NUM_POLICY_FEATURES];
        Self { value_coef, policy_coef }
    }
}

impl Default for Model {
    fn default() -> Self {
        let value_coef = from_str(DEFAULT_VALUE).unwrap();
        let policy_coef = from_str(DEFAULT_POLICY).unwrap();
        Self { value_coef, policy_coef }
    }
}
