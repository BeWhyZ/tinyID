use super::response::{ErrCode, Response};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use std::sync::Arc;

use crate::biz::{HelloWorldRepo, HelloWorldUseCase};
use crate::data::HelloWorldRepoImpl;

// 为实际使用创建类型别名
pub type HelloWorldServiceImpl = HelloWorldService<HelloWorldRepoImpl>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenIdResp {
    // id
    pub id: u64,
}

#[derive(Debug)]
pub struct HelloWorldService<R: HelloWorldRepo> {
    huc: Arc<HelloWorldUseCase<R>>,
}

impl<R: HelloWorldRepo> HelloWorldService<R> {
    pub fn new(huc: Arc<HelloWorldUseCase<R>>) -> Self {
        Self { huc }
    }

    /// 生成ID并返回Response格式
    #[tracing::instrument(skip(self))]
    pub async fn generate_id(&self) -> Response<GenIdResp> {
        let id = match self.huc.generate_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("generate id failed: {}", e);
                return Response::failed(ErrCode::InternalServerError, Some("generate id failed"));
            }
        };
        let data = GenIdResp { id };
        info!("Generated ID: {}", id);
        Response::success(Some(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biz::HelloWorldRepo;
    use crate::TinyIdError;
    use std::sync::Arc;

    // Mock repository for testing
    #[derive(Debug)]
    struct MockHelloWorldRepo {
        should_fail: bool,
        fail_with_error: Option<TinyIdError>,
        id_to_return: u64,
    }

    impl MockHelloWorldRepo {
        fn new_success(id: u64) -> Self {
            Self {
                should_fail: false,
                fail_with_error: None,
                id_to_return: id,
            }
        }

        fn new_failure(error: TinyIdError) -> Self {
            Self {
                should_fail: true,
                fail_with_error: Some(error),
                id_to_return: 0,
            }
        }
    }

    impl HelloWorldRepo for MockHelloWorldRepo {
        async fn generate_id(&self) -> Result<u64, TinyIdError> {
            if self.should_fail {
                Err(self.fail_with_error.as_ref().unwrap().clone())
            } else {
                Ok(self.id_to_return)
            }
        }
    }

    #[tokio::test]
    async fn test_generate_id_success() {
        // 创建mock repository，返回成功的ID
        let expected_id = 12345u64;
        let mock_repo = Arc::new(MockHelloWorldRepo::new_success(expected_id));
        let use_case = Arc::new(HelloWorldUseCase::new(mock_repo));
        let service = HelloWorldService::new(use_case);

        // 调用generate_id
        let response = service.generate_id().await;

        // 验证结果
        assert_eq!(response.code, ErrCode::Success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().id, expected_id);
        assert_eq!(response.msg, "操作成功");
    }

    #[tokio::test]
    async fn test_generate_id_internal_error() {
        // 创建mock repository，返回内部错误
        let mock_repo = Arc::new(MockHelloWorldRepo::new_failure(TinyIdError::InternalError(
            "database connection failed".to_string(),
        )));
        let use_case = Arc::new(HelloWorldUseCase::new(mock_repo));
        let service = HelloWorldService::new(use_case);

        // 调用generate_id
        let response = service.generate_id().await;

        // 验证错误响应
        assert_eq!(response.code, ErrCode::InternalServerError);
        assert!(response.data.is_none());
        assert_eq!(response.msg, "generate id failed");
    }

    #[tokio::test]
    async fn test_generate_id_clock_backwards_error() {
        // 创建mock repository，返回时钟回拨错误
        let mock_repo = Arc::new(MockHelloWorldRepo::new_failure(
            TinyIdError::ClockMovedBackwards(100),
        ));
        let use_case = Arc::new(HelloWorldUseCase::new(mock_repo));
        let service = HelloWorldService::new(use_case);

        // 调用generate_id
        let response = service.generate_id().await;

        // 验证错误响应
        assert_eq!(response.code, ErrCode::InternalServerError);
        assert!(response.data.is_none());
        assert_eq!(response.msg, "generate id failed");
    }

    #[tokio::test]
    async fn test_generate_id_invalid_worker_id_error() {
        // 创建mock repository，返回无效worker ID错误
        let mock_repo = Arc::new(MockHelloWorldRepo::new_failure(
            TinyIdError::InvalidWorkerId(1024),
        ));
        let use_case = Arc::new(HelloWorldUseCase::new(mock_repo));
        let service = HelloWorldService::new(use_case);

        // 调用generate_id
        let response = service.generate_id().await;

        // 验证错误响应
        assert_eq!(response.code, ErrCode::InternalServerError);
        assert!(response.data.is_none());
        assert_eq!(response.msg, "generate id failed");
    }

    #[tokio::test]
    async fn test_generate_id_multiple_calls() {
        // 测试多次调用都返回成功
        let mock_repo = Arc::new(MockHelloWorldRepo::new_success(99999));
        let use_case = Arc::new(HelloWorldUseCase::new(mock_repo));
        let service = HelloWorldService::new(use_case);

        // 多次调用
        for _ in 0..5 {
            let response = service.generate_id().await;
            assert_eq!(response.code, ErrCode::Success);
            assert!(response.data.is_some());
            assert_eq!(response.data.unwrap().id, 99999);
        }
    }

    #[test]
    fn test_gen_id_resp_serialization() {
        use serde_json;

        let resp = GenIdResp { id: 123456789 };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: GenIdResp = serde_json::from_str(&json).unwrap();

        assert_eq!(resp.id, deserialized.id);
        assert_eq!(deserialized.id, 123456789);
    }

    #[test]
    fn test_service_new() {
        let mock_repo = Arc::new(MockHelloWorldRepo::new_success(1));
        let use_case = Arc::new(HelloWorldUseCase::new(mock_repo));
        let _service = HelloWorldService::new(use_case.clone());

        // 验证service是否正确创建
        assert_eq!(Arc::strong_count(&use_case), 2); // 一个在service中，一个在测试中
    }
}
