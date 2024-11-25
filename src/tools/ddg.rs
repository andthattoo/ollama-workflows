use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    link: String,
    snippet: String,
}

pub struct RawDDGSearcher {
    client: reqwest::Client,
    base_url: String,
}

impl Default for RawDDGSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RawDDGSearcher {
    pub fn new() -> Self {
        RawDDGSearcher {
            client: reqwest::Client::new(),
            base_url: "https://duckduckgo.com".to_string(),
        }
    }

    pub async fn search(
        &self,
        query: &str,
        num_results: Option<usize>,
    ) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/html/?q={}", self.base_url, query);
        let resp = self.client.get(&url).send().await?;
        let body = resp.text().await?;
        let document = Html::parse_document(&body);

        let result_selector = Selector::parse(".web-result").unwrap();
        let result_title_selector = Selector::parse(".result__a").unwrap();
        let result_url_selector = Selector::parse(".result__url").unwrap();
        let result_snippet_selector = Selector::parse(".result__snippet").unwrap();

        let mut results: Vec<SearchResult> = document
            .select(&result_selector)
            .map(|result| {
                let title = result
                    .select(&result_title_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");
                let link = result
                    .select(&result_url_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("")
                    .trim()
                    .to_string();
                let snippet = result
                    .select(&result_snippet_selector)
                    .next()
                    .unwrap()
                    .text()
                    .collect::<Vec<_>>()
                    .join("");

                SearchResult {
                    title,
                    link,
                    snippet,
                }
            })
            .collect();

        // Limit results if specified
        if let Some(limit) = num_results {
            results.truncate(limit);
        }

        // Serialize to JSON string
        Ok(serde_json::to_string(&results)?)
    }
}
