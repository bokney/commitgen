
use std::env;
use colored::*;
use clap::Parser;
use dotenv::dotenv;
use serde_json::Value;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};

#[async_trait::async_trait]
pub trait LLMClient {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String>;
}

pub struct GeminiClient {
    api_key: String,
    endpoint: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        );
        Self { api_key, endpoint }
    }
}

#[async_trait::async_trait]
impl LLMClient for GeminiClient {
    async fn generate(&self, prompt: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let resp = client
            .post(&self.endpoint)
            .header("x-goog-api-key", &self.api_key)
            .json(&serde_json::json!({
                "contents": [{ "parts": [{ "text": prompt }] }],
                "generationConfig": { "temperature": 0.0, "maxOutputTokens": 4096 }
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await?;
            return Err(anyhow::anyhow!(
                "Gemini API returned HTTP {}:\n{}", status, text
            ));
        }

        let v: Value = resp.json().await?;

        let message = v
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.trim().to_string());

        message.ok_or_else(|| anyhow::anyhow!("Failed to extract message. Response: {}", v))
    }
}

#[derive(Parser)]
#[command(author, version, about = "Generate Git commit messages with Gemini API")]
struct CLI {
    description: String,
    
    #[arg(short, long, default_value = "conventional commit")]
    style: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set in .env");

    let args = CLI::parse();

    let prompt = format!(
        "You are an expert programmer writing a git commit message.\n\
        Your task is to generate a single, git commit message in the '{style}' style for the following change description.\n\n\
        VERY IMPORTANT: Your entire response must be only the commit message itself. Do not include any surrounding text, explanations, apologies, or markdown formatting like ```.\n\n\
        Change Description: \"{description}\"",
        style = args.style,
        description = args.description
    );

    let llm: Box<dyn LLMClient> = Box::new(GeminiClient::new(api_key));

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} {msg}")?,
    );
    spinner.set_message("Generating commit message...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let result = llm.generate(&prompt).await;

    spinner.finish_and_clear();
    match result {
        Ok(message) => {
            println!();
            println!("{}", message.cyan());
            println!();
        }
        Err(e) => {
            eprintln!("\n{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }

    Ok(())
}

