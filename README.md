
# 🧠 CommitGen

**CommitGen** is a CLI tool for generating Git commit messages using natural language prompts and Google’s Gemini API, with a testable trait-based architecture.

## 📆 Features

* 🖍️ Transforms human-readable descriptions into conventional or gitmoji commit messages
* ⚡ Fast, async implementation with `tokio` and `reqwest`
* 🎛️ Supports custom commit styles (e.g., `conventional commit`, `gitmoji`)
* 🛠️ Modular design using an `LLMClient` trait for easy testing and extensibility
* ✨ Polished UX with a live spinner and coloured output via `indicatif` and `colored`

## 🚀 Usage

```bash
cargo run -- "refactored the project to use trait-based LLM abstraction"
```

Optional flags:

```bash
cargo run -- "add login support" --style "gitmoji"
```

## 🔧 Installation

1. Clone the repo:

   ```bash
   git clone https://github.com/bokney/commitgen.git
   cd commitgen
   ```

2. Set up the Gemini API key:

   ```bash
   echo 'GEMINI_API_KEY="your_api_key_here"' > .env
   ```

3. Run the app:

   ```bash
   cargo run -- "fixed the bug that was making all the fish big and fast"
   ```

## 📁 Project Structure

* `main.rs`: CLI parsing and core application loop
* `LLMClient` trait: Abstract interface for LLM providers
* `GeminiClient`: Handles Gemini API interaction

## 🧪 Testing

You can inject mock implementations of `LLMClient` for unit tests without making real HTTP calls.

## 📜 License

MIT

