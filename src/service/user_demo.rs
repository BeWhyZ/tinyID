pub mod user_demo_srv {
    tonic::include_proto!("id_generator.v1");
}
use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::error;

use crate::biz::user_demo::UserDemoUseCase;
use crate::data::UserDemoRepoImpl;

pub use user_demo_srv::user_demo_server::UserDemo;
pub use user_demo_srv::{GetUserRequest, GetUserResponse};

#[derive(Debug, Clone)]
pub struct UserDemoSrvImpl {
    huc: Arc<UserDemoUseCase<UserDemoRepoImpl>>,
}

impl UserDemoSrvImpl {
    pub fn new(huc: Arc<UserDemoUseCase<UserDemoRepoImpl>>) -> Self {
        Self { huc }
    }
}

#[tonic::async_trait]
impl UserDemo for UserDemoSrvImpl {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let resp = self.huc.get_user(request.get_ref().id).await;
        match resp {
            Ok(user) => Ok(Response::new(GetUserResponse {
                name: user.name,
                age: user.age as i32,
                id: user.id,
            })),
            Err(e) => {
                error!("get user failed: {}", e);
                Err(Status::internal("get user failed"))
            }
        }
    }
}
