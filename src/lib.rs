pub mod http;
pub mod macros;
pub mod sites;

pub struct TorrentResult {
    pub title: String,
    pub magnet: String,
}

#[async_trait::async_trait]
pub trait TorrentSite {
    async fn search(&self, keyword: &str) -> anyhow::Result<Vec<TorrentResult>>;
}
