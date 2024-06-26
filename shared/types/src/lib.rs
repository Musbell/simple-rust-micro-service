use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Data {
    PostData(Post),
    CommentData(Comment),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: Option<Uuid>,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EventType {
    PostCreated,
    CommentCreated,
    CommentModerated,
    CommentUpdated,
    Other
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub data: Data,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CommentStatus {
    Pending,
    Approved,
    Rejected,
}

impl Default for CommentStatus {
    fn default() -> Self {
        CommentStatus::Pending
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    pub id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub content: String,
    pub status: Option<CommentStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModerateComment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub status: CommentStatus,
}
