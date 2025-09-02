pub mod hello_world;
mod rpc;

pub use hello_world::{HelloWorldRepoImpl, IDGenerator};

pub mod rpc_client {
    tonic::include_proto!("id_generator.v1");
}

pub use rpc::new_id_generator_client;
