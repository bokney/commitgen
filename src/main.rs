
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use dotenv::dotenv;
use std::env;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about = "Generate Git commit messages with Gemini API")]
struct CLI {
    description: String,
    
    #[arg(short, long, default_value = "conventional commit")]
    style: String,
}

async fn generate_commit_message(
    api_key: &str,
    prompt: &str,
) -> anyhow::Result<String> {
    let endpoint = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .json(&serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": 0.0,
                "maxOutputTokens": 4096
            }
        }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await?;
        return Err(anyhow::anyhow!(
            "Gemini API returned HTTP {}:\n{}",
            status,
            text
        ));
    }

    let v: serde_json::Value = resp.json().await?;

    let message = v
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.trim().to_string());

    message.ok_or_else(|| {
        anyhow::anyhow!(
            "Could not extract message from API response. Full response:\n{}",
            v
        )
    })
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

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} {msg}")?,
    );
    spinner.set_message("Generating commit message...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let result = generate_commit_message(&api_key, &prompt).await;

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

