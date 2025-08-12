pub struct ServerConfig {
    pub addr: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn new(addr: String, port: u16) -> Self {
        Self { addr, port }
    }
}
