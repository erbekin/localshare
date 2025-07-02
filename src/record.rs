use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileRecord {
    // unique id and also filename
    pub id: Uuid,
    // original name from user to display purposes
    pub name: String,
    // owner can be null
    pub by: Option<String>,
    // datetime
    pub uploaded_at: DateTime<Utc>,
    // description string
    pub description: String,
    // content type to response appropritetly
    pub content_type: Option<String>,
}

