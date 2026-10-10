# Serverless LLM-Based Chatbot with Spin and Rust

A lightweight, serverless conversational chatbot built with [Spin](https://github.com/spinframework/spin), Rust, and the WebAssembly Component Model (`wasm32-wasip2`).

## Architecture

This project demonstrates how to bridge the gap between stateless serverless WebAssembly components and stateful conversational AI:

- **Stateless Execution:** The Spin component boots instantly (< 1 ms) to process each request and shuts down immediately after responding.
- **Session Memory in Key-Value Storage:** Conversation history is persisted across turns in Spin's built-in key-value store, keyed by `session_id`.
- **Frontier LLM Integration via Outbound HTTP:** The component constructs OpenAI-compatible JSON payloads and dispatches them to frontier models (OpenAI, Groq, OpenRouter, etc.) using `spin_sdk::http::send`.
- **Zero Third-Party AI Frameworks:** Implemented in ~150 lines of idiomatic Rust with Serde, compiling to a tiny ~625 KB Wasm binary.

## Configuration

The application uses Spin application variables configured in `spin.toml`:

| Variable | Default Value | Description |
| :--- | :--- | :--- |
| `openai_api_key` | `""` | API key for the LLM provider (passed as a Bearer token). |
| `openai_base_url` | `https://api.openai.com/v1` | Base URL for the OpenAI-compatible REST API. |
| `openai_model` | `gpt-4o-mini` | The model identifier to query. |

## Build and Run with OpenAI

```bash
spin up --build --variable openai_api_key="sk-your-openai-api-key"
```
## Send a Chat Message (`POST /chat`)

Send a message in a conversation session:

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "session-101",
    "message": "Hello! I am planning a trip to Tokyo. What are 3 must-visit areas?"
  }'
```

Response:

```json
{
  "session_id": "session-101",
  "reply": "1. Shibuya – Famous for the iconic crossing...\n2. Shinjuku – Vibrant nightlife...\n3. Asakusa – Rich in traditional culture..."
}
```

Follow up in the same session to verify multi-turn conversational memory:

```bash
curl -X POST http://localhost:3000/chat \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "session-101",
    "message": "Which of those three is closest to ancient temples?"
  }'
```

Response:

```json
{
  "session_id": "session-101",
  "reply": "Asakusa! It is home to Sensō-ji, Tokyo's oldest and most famous Buddhist temple..."
}
```

## Retrieve Conversation History

Inspect the full conversation history preserved in the key-value store:

```bash
curl "http://localhost:3000/history?session_id=session-101"
```

## Clear Conversation History

Reset the conversation memory for a session:

```bash
curl -X DELETE "http://localhost:3000/history?session_id=session-101"
```
