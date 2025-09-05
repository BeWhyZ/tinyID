use anyhow::Result;
use tracing::instrument;

use crate::biz::{User, UserRepo};
use crate::error::UserError;

#[derive(Debug, Clone)]
pub struct UserRepoImpl {}

impl UserRepoImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl UserRepo for UserRepoImpl {
    #[instrument(skip(self))]
    async fn get_user(&self, id: u64) -> Result<User, UserError> {
        Ok(User::new(id, "test".to_string(), "test".to_string(), 18))
    }
}
