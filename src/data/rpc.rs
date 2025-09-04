use tonic::transport::Channel;

use super::rpc_client::id_generator_service_client::IdGeneratorServiceClient;
use crate::config::IdGeneratorRpcConfig;

pub fn new_id_generator_client(
    cfg: IdGeneratorRpcConfig,
) -> Result<IdGeneratorServiceClient<Channel>, Box<dyn std::error::Error>> {
    let endpoints = cfg
        .rpc_cfg
        .addr
        .into_iter()
        .map(|a| Channel::from_shared(a).unwrap());
    let channel = Channel::balance_list(endpoints);
    let client: IdGeneratorServiceClient<Channel> = IdGeneratorServiceClient::new(channel);
    Ok(client)
}
