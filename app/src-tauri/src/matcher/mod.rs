pub mod exact;
pub mod similar;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchPair {
    pub new_file_id: String,
    pub existing_file_id: String,
    pub new_file_path: String,
    pub existing_file_path: String,
    pub algorithm: String,
    pub similarity_score: f64,
    pub match_type: String,
}
