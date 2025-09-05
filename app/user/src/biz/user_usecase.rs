use std::sync::Arc;

use anyhow::Result;
use tracing::instrument;

use super::models::User;
use crate::error::UserError;

pub trait UserRepo: Send + Sync + std::fmt::Debug {
    fn get_user(
        &self,
        id: u64,
    ) -> impl std::future::Future<Output = Result<User, UserError>> + Send;
}

/// 用户业务逻辑用例
#[derive(Debug)]
pub struct UserUseCase<R: UserRepo> {
    user_repo: Arc<R>,
}

impl<R: UserRepo> UserUseCase<R> {
    pub fn new(user_repo: Arc<R>) -> Self {
        Self { user_repo }
    }

    #[instrument(skip(self))]
    pub async fn get_user(&self, id: u64) -> Result<User, UserError> {
        if id == 0 {
            return Err(UserError::InvalidData("user id cannot be 0".to_string()));
        }

        self.user_repo.get_user(id).await
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::data::InMemoryUserRepository;

//     #[tokio::test]
//     async fn test_get_user_success() {
//         let repo = Arc::new(InMemoryUserRepository::new());
//         let usecase = UserUseCase::new(repo);

//         let user = usecase.get_user(1).await.unwrap();
//         assert_eq!(user.name, "Alice");
//     }

//     #[tokio::test]
//     async fn test_get_user_invalid_id() {
//         let repo = Arc::new(InMemoryUserRepository::new());
//         let usecase = UserUseCase::new(repo);

//         let result = usecase.get_user(0).await;
//         assert!(matches!(result, Err(UserError::InvalidData(_))));
//     }
// }
