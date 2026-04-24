use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KccScore {
    pub total: u32,
    pub factors: Vec<KccFactor>,
    pub classification: KccClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KccFactor {
    pub name: String,
    pub points: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KccClassification {
    Kcc,       // score >= 8
    Important, // score 5-7
    Standard,  // score < 5
}

impl KccClassification {
    pub fn as_str(&self) -> &str {
        match self {
            KccClassification::Kcc => "kcc",
            KccClassification::Important => "important",
            KccClassification::Standard => "standard",
        }
    }
}

impl std::fmt::Display for KccClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
