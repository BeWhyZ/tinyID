use tracing::instrument;

use crate::biz::user_demo::{User, UserDemoRepo};
use crate::TinyIdError;

#[derive(Debug, Clone)]
pub struct UserDemoRepoImpl {}

impl UserDemoRepoImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl UserDemoRepo for UserDemoRepoImpl {
    #[instrument(skip(self))]
    async fn get_user(&self, id: u64) -> Result<User, TinyIdError> {
        Ok(User {
            id,
            name: "test".to_string(),
            age: 18,
        })
    }
}
