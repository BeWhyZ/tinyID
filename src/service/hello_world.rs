use super::response::Response;
use rand::random;
use serde::{Deserialize, Serialize};
use tracing::info;

use std::sync::Arc;

use crate::biz::{HelloWorldRepo, HelloWorldUseCase};
use crate::data::HelloWorldRepoImpl;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenIdResp {
    // id
    pub id: u64,
}

#[derive(Debug)]
pub struct HelloWorldService {
    huc: Arc<HelloWorldUseCase<HelloWorldRepoImpl>>,
}

impl HelloWorldService {
    pub fn new(huc: Arc<HelloWorldUseCase<HelloWorldRepoImpl>>) -> Self {
        Self { huc }
    }

    /// 生成ID并返回Response格式
    #[tracing::instrument]
    pub async fn generate_id(&self) -> Response<GenIdResp> {
        let id = self.huc.generate_id().await;
        let data = GenIdResp { id };
        info!("Generated ID: {}", id);
        Response::success(Some(data))
    }
}
