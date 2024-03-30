use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: Option<Uuid>,
    pub title: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub data: Post,
}
