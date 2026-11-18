use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;
use url::Url;

#[derive(Clone)]
pub struct GatewayState {
    pub upstream: Url,
    pub client: Client,
    pub permits: Arc<Semaphore>,
}

impl GatewayState {
    pub fn new(upstream: Url) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::none())
            .pool_max_idle_per_host(16)
            .build()?;
        Ok(Self {
            upstream,
            client,
            permits: Arc::new(Semaphore::new(4)),
        })
    }
}
