use crate::http::{build_client, build_detail_url, build_search_url, MAX_DETAIL_LINKS};
use crate::{TorrentResult, TorrentSite};
use futures::future::join_all;
use scraper::{Html, Selector};
use tokio::task::spawn_blocking;

/// TorrentTop struct holds base_url
pub struct TorrentTop {
    pub base_url: String,
}

impl TorrentTop {
    /// Create a new instance of TorrentTop with given base_url
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Parse the HTML document and extract (title, href) links
    pub fn parse_search_document(document: &Html) -> Vec<(String, String)> {
        let a_selector = Selector::parse("div.flex-auto.px-2.truncate a").unwrap();

        document
            .select(&a_selector)
            .filter_map(|a_tag| {
                let href = a_tag.value().attr("href")?;
                if href.starts_with("/torrent/") {
                    let title = a_tag.value().attr("title").unwrap_or("").to_string();
                    Some((title, href.to_string()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Parse the magnet URL from detail page HTML
    pub fn parse_magnet_document(document: &Html) -> Option<String> {
        let magnet_selector = Selector::parse("a[href^=\"magnet:\"]").ok()?;
        document
            .select(&magnet_selector)
            .next()?
            .value()
            .attr("href")
            .map(|s| s.to_string())
    }
}

#[async_trait::async_trait]
impl TorrentSite for TorrentTop {
    async fn search(&self, keyword: &str) -> anyhow::Result<Vec<TorrentResult>> {
        let client = build_client()?;
        let search_url = build_search_url(&self.base_url, keyword)?;
        let resp = client.get(search_url).send().await?.text().await?;

        let link_list = spawn_blocking({
            let resp = resp.clone();
            move || -> anyhow::Result<Vec<(String, String)>> {
                let document = Html::parse_document(&resp);
                Ok(Self::parse_search_document(&document))
            }
        })
        .await??;

        let futures = link_list
            .into_iter()
            .filter_map(|(title, href)| {
                let detail_url = build_detail_url(&self.base_url, &href).ok()?;
                Some((title, detail_url))
            })
            .take(MAX_DETAIL_LINKS)
            .map(|(title, detail_url)| {
                let client = client.clone();
                async move {
                    let detail_resp = client
                        .get(detail_url)
                        .send()
                        .await
                        .ok()?
                        .text()
                        .await
                        .ok()?;

                    let magnet = spawn_blocking(move || -> Option<String> {
                        let detail_doc = Html::parse_document(&detail_resp);
                        Self::parse_magnet_document(&detail_doc)
                    })
                    .await
                    .ok()??;

                    Some(TorrentResult { title, magnet })
                }
            });

        let results = join_all(futures).await;
        let results: Vec<TorrentResult> = results.into_iter().flatten().collect();

        Ok(results)
    }
}
