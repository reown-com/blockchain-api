#[derive(Debug, Clone)]
pub struct Config {
    pub chains: Vec<ChainConfig>,
}

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub caip2: String,
    pub name: String,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub url: String,
}
