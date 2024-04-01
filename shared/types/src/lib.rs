use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Data {
    PostData(Post),
    CommentData(Comment),
    ModerationData(Comment),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: Option<Uuid>,
    pub title: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event_type: String,
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
