use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "userId")]
    pub id: String,
    pub coins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(rename = "userId")]
    pub id: String,
    pub pronouns: Option<String>,
    #[serde(rename = "dateOfBirth")]
    pub dob: Option<String>,
}
