use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: u64, name: String, email: String, age: i32) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            email,
            age,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<User> for shared::proto::user::User {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            age: user.age,
            created_at: user.created_at.timestamp(),
            updated_at: user.updated_at.timestamp(),
        }
    }
}
