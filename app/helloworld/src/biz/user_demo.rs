use std::sync::Arc;

use shared::proto::user::User;
use tracing::instrument;

use crate::TinyIdError;

pub trait UserDemoRepo: Send + Sync + std::fmt::Debug {
    fn get_user(
        &self,
        id: u64,
    ) -> impl std::future::Future<Output = Result<User, TinyIdError>> + Send;
}

#[derive(Debug, Clone)]
pub struct UserDemoUseCase<R: UserDemoRepo> {
    hrepo: Arc<R>,
}

impl<R: UserDemoRepo> UserDemoUseCase<R> {
    pub fn new(hrepo: Arc<R>) -> Self {
        Self { hrepo }
    }
}

impl<R: UserDemoRepo> UserDemoUseCase<R> {
    #[instrument(skip(self))]
    pub async fn get_user(&self, id: u64) -> Result<User, TinyIdError> {
        self.hrepo.get_user(id).await
    }
}
