use crate::biz::HelloWorldRepo;
use rand::random;

#[derive(Debug)]
pub struct HelloWorldRepoImpl {}

impl HelloWorldRepo for HelloWorldRepoImpl {
    async fn generate_id(&self) -> u64 {
        random::<u64>()
    }
}

impl HelloWorldRepoImpl {
    pub fn new() -> Self {
        Self {}
    }
}
