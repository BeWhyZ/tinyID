use std::sync::Arc;

// use anyhow::{Context, Result};
use tracing::instrument;

use crate::TinyIdError;

pub trait HelloWorldRepo: Send + Sync + std::fmt::Debug {
    fn generate_id(&self) -> impl std::future::Future<Output = Result<u64, TinyIdError>> + Send;
}

#[derive(Debug, Clone)]
pub struct HelloWorldUseCase<R: HelloWorldRepo> {
    hrepo: Arc<R>,
}

impl<R: HelloWorldRepo> HelloWorldUseCase<R> {
    pub fn new(hrepo: Arc<R>) -> Self {
        Self { hrepo }
    }

    #[instrument(skip(self))]
    pub async fn generate_id(&self) -> Result<u64, TinyIdError> {
        self.hrepo.generate_id().await
    }
}
