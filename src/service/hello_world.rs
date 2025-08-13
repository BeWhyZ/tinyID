use super::response::Response;
use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenIdResp {
    // id
    pub id: u64,
}

pub struct HelloWorldService {}

impl HelloWorldService {
    pub fn new() -> Self {
        Self {}
    }

    /// 生成ID并返回Response格式
    pub async fn generate_id(&self) -> Response<GenIdResp> {
        let id = random::<u64>();
        let data = GenIdResp { id };
        Response::success(Some(data))
    }
}
