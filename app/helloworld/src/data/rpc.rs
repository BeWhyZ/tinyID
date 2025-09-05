use shared::config::UserRpcConfig;
use shared::proto::user::user_demo_client::UserDemoClient;
use tonic::transport::Channel;

pub fn new_user_client(
    cfg: UserRpcConfig,
) -> Result<UserDemoClient<Channel>, Box<dyn std::error::Error>> {
    let endpoints = cfg
        .rpc_cfg
        .addr
        .into_iter()
        .map(|a| Channel::from_shared(a).unwrap());
    let channel = Channel::balance_list(endpoints);
    let client: UserDemoClient<Channel> = UserDemoClient::new(channel);
    Ok(client)
}
