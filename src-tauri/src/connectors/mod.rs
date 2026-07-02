pub mod add;
pub mod bluesky;
pub mod rss;
pub mod website;

use std::sync::OnceLock;

use async_trait::async_trait;

use crate::{AppState, models::Article};

#[async_trait]
pub trait FeedConnector: Send + Sync {
    fn feed_type(&self) -> &'static str;
    async fn fetch_articles(
        &self,
        url: &str,
        state: &AppState,
    ) -> Result<(String, String, Vec<Article>), String>;

    async fn refresh(&self, feed_url: &str, feed_id: i64, state: &AppState) -> Result<i64, String>;
}

pub struct Registry {
    connectors: Vec<Box<dyn FeedConnector>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            connectors: Vec::new(),
        }
    }

    pub fn register(&mut self, connector: Box<dyn FeedConnector>) {
        self.connectors.push(connector);
    }

    pub async fn detect_and_add(
        &self,
        url: &str,
        folder_id: Option<i64>,
        state: &AppState,
    ) -> Result<i64, String> {
        for connector in &self.connectors {
            if let Ok((title, feed_url, articles)) = connector.fetch_articles(url, state).await {
                return add::add_feed_with_articles(
                    &title,
                    &feed_url,
                    connector.feed_type(),
                    articles,
                    folder_id,
                    state,
                )
                .await;
            }
        }
        Err("Could not add feed: unsupported URL".to_string())
    }

    pub async fn refresh(
        &self,
        feed_type: &str,
        feed_url: &str,
        feed_id: i64,
        state: &AppState,
    ) -> Result<i64, String> {
        for connector in &self.connectors {
            if connector.feed_type() == feed_type {
                return connector.refresh(feed_url, feed_id, state).await;
            }
        }
        Err(format!("Unknown feed type: {}", feed_type))
    }
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get_or_init(|| {
        let mut reg = Registry::new();
        reg.register(Box::new(bluesky::BlueskyConnector));
        reg.register(Box::new(rss::RssConnector));
        reg.register(Box::new(website::WebsiteConnector));
        reg
    })
}
