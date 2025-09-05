use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument};

use shared::proto::user::{
    user_demo_server::UserDemo as UserServiceTrait, GetUserRequest, GetUserResponse,
};

use crate::biz::UserUseCase;
use crate::data::UserRepoImpl;

#[derive(Debug, Clone)]
pub struct UserDemoSrvImpl {
    huc: Arc<UserUseCase<UserRepoImpl>>,
}

impl UserDemoSrvImpl {
    pub fn new(huc: Arc<UserUseCase<UserRepoImpl>>) -> Self {
        Self { huc }
    }
}

#[tonic::async_trait]
impl UserServiceTrait for UserDemoSrvImpl {
    #[instrument(skip(self))]
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let resp = self.huc.get_user(request.get_ref().id).await;
        match resp {
            Ok(user) => Ok(Response::new(GetUserResponse {
                user: Some(user.into()),
            })),
            Err(e) => {
                error!("get user failed: {}", e);
                Err(Status::internal("get user failed"))
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::data::InMemoryUserRepository;

//     #[tokio::test]
//     async fn test_get_user_success() {
//         let repo = Arc::new(InMemoryUserRepository::new());
//         let usecase = Arc::new(UserUseCase::new(repo));
//         let service = UserService::new(usecase);

//         let request = Request::new(GetUserRequest { id: 1 });
//         let response = service.get_user(request).await.unwrap();

//         let user = response.into_inner().user.unwrap();
//         assert_eq!(user.name, "Alice");
//         assert_eq!(user.email, "alice@example.com");
//     }
// }
