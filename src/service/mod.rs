use tonic::{transport::Server, Request, Response, Status};

pub mod hello_world;
pub mod response;

// rpc service
pub use hello_world::{HelloWorldService, HelloWorldServiceImpl};
pub mod id_generator {
    tonic::include_proto!("id_generator.v1");
}
pub use id_generator::id_generator_service_server::IdGeneratorService;
