
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
/// Represents a record of a file uploaded to the system.
///
/// Each `FileRecord` contains metadata about the file, including a unique identifier,
/// display name, optional owner, upload timestamp, description, and optional content type.
///
/// # Fields
/// - `id`: Unique identifier for the file, also used as the filename.
/// - `name`: Original name of the file provided by the user, used for display purposes.
/// - `by`: Optional owner of the file. Can be `None` if the owner is not specified.
/// - `uploaded_at`: Timestamp indicating when the file was uploaded.
/// - `description`: Description of the file.
/// - `content_type`: Optional MIME type of the file, used to determine how to respond to requests.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileRecord {
    pub id: Uuid,
    pub name: String,
    pub by: Option<String>,
    pub uploaded_at: DateTime<Utc>,
    pub description: String,
    pub content_type: Option<String>,
}

