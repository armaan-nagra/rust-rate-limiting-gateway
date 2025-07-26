# 🚦 High Performance API Rate Limiting Proxy

A **self-imposed rate limiting proxy** that helps you control your own API usage and costs. Perfect for developers who want to set stricter limits than APIs provide, protect against accidental overuse, and keep API keys secure.

## 🎯 Why Use This?

**Problem**: APIs often have generous rate limits (e.g., OpenAI allows 3,500 requests/minute), but you want stricter control to:
- 💰 **Prevent bill shock** from accidental loops or bugs
- 🎯 **Self-impose limits** stricter than the API's defaults  
- 🔒 **Protect API keys** from exposure in frontend code
- 📊 **Monitor usage** across different projects/users

**Solution**: This proxy sits between your code and any API, letting you set your own limits.

## 🏗️ How It Works

```
Your App → Rate Limiter → External API (OpenAI, Stripe, etc.)
```

**When you hit your self-imposed limit**: Returns HTTP 429 instead of forwarding to the expensive API.

**Tech Stack**: Rust + Axum + Redis for high performance (~1ms overhead, >10k req/sec)

## 🚀 Quick Start

```bash
# 1. Start Redis
redis-server

# 2. Configure your API
export TARGET_API_URL="https://api.openai.com/v1"
export TARGET_API_KEY="sk-your-openai-key"
export RATE_LIMIT="60"  # requests per minute

# 3. Run the proxy
cargo run

# 4. Get a token
TOKEN=$(curl -s -X POST http://127.0.0.1:3000/register | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

# 5. Use the proxy instead of direct API calls
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3000/api/models
```

## 🚀 Try the Demo

```bash
# Terminal 1: Start with demo settings
export TARGET_API_URL="https://httpbin.org"
export RATE_LIMIT="5"
cargo run

# Terminal 2: See it in action
./demo.sh
```

The demo makes 8 requests, shows ~5 succeeding and ~3 getting rate limited.

## 🔧 Configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `TARGET_API_URL` | API base URL | `https://api.openai.com/v1` |
| `TARGET_API_KEY` | API key (optional) | `sk-...` |
| `RATE_LIMIT` | Requests per minute | `60` |

## 🏗️ Technical Highlights

- **Rust + Axum**: Memory-safe, zero-cost abstractions, async/await
- **Redis**: Atomic rate limiting, distributed across instances
- **Streaming Proxy**: Constant memory usage for any payload size
- **Stateless Design**: Horizontally scalable behind load balancer

## 💡 Benefits

- **Cost Control**: Never accidentally exceed budgets
- **Security**: API keys never exposed to frontend
- **Production Ready**: <50MB memory, 1-2ms latency overhead
- **Scalable**: Handle 10k+ concurrent connections

## ❓ Troubleshooting

- **Server won't start**: Check Redis is running (`redis-server`)
- **429 errors**: Working as intended! Your rate limit is protecting you
- **API calls failing**: Verify `TARGET_API_URL` and use `/api/` prefix

## 🤝 Contributing

This is a learning project! Feel free to suggest improvements, submit PRs, or open issues. All contributions are welcome!

## 📄 License

MIT License - use it however you want!
