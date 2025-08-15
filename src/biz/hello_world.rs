use std::sync::Arc;

pub trait HelloWorldRepo: Send + Sync {
    fn generate_id(&self) -> impl std::future::Future<Output = u64> + Send;
}

#[derive(Debug)]
pub struct HelloWorldUseCase<R: HelloWorldRepo> {
    hrepo: Arc<R>,
}

impl<R: HelloWorldRepo> HelloWorldUseCase<R> {
    pub fn new(hrepo: Arc<R>) -> Self {
        Self { hrepo }
    }

    pub async fn generate_id(&self) -> u64 {
        self.hrepo.generate_id().await
    }
}
