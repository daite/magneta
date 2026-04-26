use anyhow::{anyhow, Context};
use reqwest::{Client, Url};
use std::time::Duration;

pub const MAX_DETAIL_LINKS: usize = 32;
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

pub fn build_client() -> anyhow::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .context("failed to build HTTP client")
}

pub fn validate_base_url(base_url: &str) -> anyhow::Result<Url> {
    let url = Url::parse(base_url).with_context(|| format!("invalid base_url: {base_url}"))?;

    if url.scheme() != "https" {
        return Err(anyhow!("base_url must use https: {base_url}"));
    }

    if url.host_str().is_none() {
        return Err(anyhow!("base_url must include a host: {base_url}"));
    }

    Ok(url)
}

pub fn build_search_url(base_url: &str, keyword: &str) -> anyhow::Result<Url> {
    let base = validate_base_url(base_url)?;
    let mut url = base.join("/search/index")?;
    url.query_pairs_mut().append_pair("keywords", keyword);
    Ok(url)
}

pub fn build_detail_url(base_url: &str, href: &str) -> anyhow::Result<Url> {
    let base = validate_base_url(base_url)?;
    let detail = base.join(href)?;

    if !same_origin(&base, &detail) {
        return Err(anyhow!("detail link points to a different origin"));
    }

    Ok(detail)
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_search_url_encodes_keyword() {
        let url = build_search_url("https://example.com", "a b&c").unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.com/search/index?keywords=a+b%26c"
        );
    }

    #[test]
    fn validate_base_url_rejects_plain_http() {
        assert!(validate_base_url("http://example.com").is_err());
    }

    #[test]
    fn build_detail_url_allows_same_origin_relative_paths() {
        let url = build_detail_url("https://example.com", "/torrent/123").unwrap();
        assert_eq!(url.as_str(), "https://example.com/torrent/123");
    }

    #[test]
    fn build_detail_url_rejects_cross_origin_links() {
        assert!(
            build_detail_url("https://example.com", "https://evil.example/torrent/123").is_err()
        );
    }
}
