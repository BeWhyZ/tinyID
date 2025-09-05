use std::sync::Arc;

use axum::extract::Query;
use axum::response::Json;
use serde::{Deserialize, Serialize};
use shared::proto::id_generator::id_generator_service_server::IdGeneratorService;
use shared::proto::id_generator::{GenerateIdRequest, GenerateIdResponse};
use tonic::{Request, Response as TResponse, Status};
use tracing::{error, info};

use super::response::{ErrCode, Response};
use crate::biz::{HelloWorldRepo, HelloWorldUseCase, UserDemoRepo, UserDemoUseCase};
use crate::data::HelloWorldRepoImpl;

// 为实际使用创建类型别名
pub type HelloWorldServiceImpl = HelloWorldService<HelloWorldRepoImpl, HelloWorldRepoImpl>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenIdResp {
    // id
    pub id: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct GetUserReq {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUserResp {
    pub id: u64,
    pub name: String,
    pub age: i32,
    pub email: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub struct HelloWorldService<R, U>
where
    R: HelloWorldRepo,
    U: UserDemoRepo,
{
    huc: Arc<HelloWorldUseCase<R>>,
    uuc: Arc<UserDemoUseCase<U>>,
}

impl<R: HelloWorldRepo, U: UserDemoRepo> HelloWorldService<R, U> {
    pub fn new(huc: Arc<HelloWorldUseCase<R>>, uuc: Arc<UserDemoUseCase<U>>) -> Self {
        Self { huc, uuc }
    }

    /// 生成ID并返回Response格式  
    #[tracing::instrument(skip(self), fields(operation = "generate_id"))]
    pub async fn generate_id(&self) -> Json<Response<GenIdResp>> {
        let id = match self.huc.generate_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("generate id failed: {}", e);
                return Json(Response::failed(
                    ErrCode::InternalServerError,
                    Some("generate id failed"),
                ));
            }
        };
        let data = GenIdResp { id };
        info!("Generated ID: {}", id);

        Json(Response::success(Some(data)))
    }

    /// 获取用户信息
    #[tracing::instrument(
        skip(self),
        fields(
            operation = "get_user",
            user_id = %req.id,
        )
    )]
    pub async fn get_user(&self, Query(req): Query<GetUserReq>) -> Json<Response<GetUserResp>> {
        let user = match self.uuc.get_user(req.id).await {
            Ok(user) => user,
            Err(e) => {
                error!("generate id failed: {}", e);
                return Json(Response::failed(
                    ErrCode::InternalServerError,
                    Some("generate id failed"),
                ));
            }
        };
        let data = GetUserResp {
            id: user.id,
            name: user.name,
            age: user.age,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };
        info!("Get user: {:?}", data);
        Json(Response::success(Some(data)))
    }
}

#[tonic::async_trait]
impl IdGeneratorService for HelloWorldService<HelloWorldRepoImpl, HelloWorldRepoImpl> {
    /// gRPC生成ID接口
    #[tracing::instrument(skip(self), fields(operation = "grpc_generate_id", protocol = "grpc"))]
    async fn generate_id(
        &self,
        _request: Request<GenerateIdRequest>,
    ) -> Result<TResponse<GenerateIdResponse>, Status> {
        let id_resp = self.huc.generate_id().await;
        match id_resp {
            Ok(id) => return Ok(TResponse::new(GenerateIdResponse { id: id })),
            Err(e) => {
                error!("generate id failed: {}", e);
                return Err(Status::internal("generate id failed"));
            }
        }
    }
}
