use async_trait::async_trait;
use ollama_rs::generation::functions::tools::Tool;
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::error::Error;

pub struct Jina {}

#[async_trait]
impl Tool for Jina {
    fn name(&self) -> String {
        "jina_web_scraper".to_string()
    }

    fn description(&self) -> String {
        "Scrapes text content from websites using Jina API and returns in readable format."
            .to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "website": {
                    "type": "string",
                    "description": "The URL of the website to scrape"
                }
            },
            "required": ["website"]
        })
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error>> {
        let website = input["website"].as_str().ok_or("Website URL is required")?;
        let token = env::var("JINA_API_KEY");
        //if no token, send req w/o token
        if token.is_err() {
            let url = format!("https://r.jina.ai/{}", website);
            let client = Client::new();
            let response = client.get(&url).send().await?;

            let result = response.text().await?;
            return Ok(result);
        } else {
            let bearer = format!("Bearer {}", token.unwrap());
            let url = format!("https://r.jina.ai/{}", website);
            let client = Client::new();
            let response = client
                .get(&url)
                .header("Authorization", bearer)
                //.header("X-With-Generated-Alt", "false")
                //.header("X-With-Images-Summary", "false")
                //.header("X-With-Links-Summary", "false")
                .send()
                .await?;
            let result = response.text().await?;
            Ok(result)
        }
    }
}
