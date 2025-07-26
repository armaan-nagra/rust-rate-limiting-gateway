[![Rust](https://img.shields.io/badge/Rust-Performance-orange?logo=rust)](https://www.rust-lang.org/)
[![Redis](https://img.shields.io/badge/Redis-Caching-red?logo=redis)](https://redis.io/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> A blazing-fast, self-hosted API gateway with **token-based authentication** and **Redis-backed rate limiting**, built in **Rust**. Inspired by Stripe/GitHub/OpenAI’s API infra.

---

## 📌 Overview

This project is a minimal, production-style API middleware designed to:

- 🔐 Issue and verify API tokens
- 🚦 Enforce per-user rate limits (e.g. 10 req/min)
- ⚡ Maintain sub-millisecond response times using Rust + Redis

---

## ✨ Features

- ✅ Token-based authentication (`Bearer <token>`)
- 🚀 Fixed-window rate limiting with Redis counters
- 🧵 Concurrency via `tokio` and async handlers
- 📈 Ready for benchmarking (`wrk`, `hey`, etc.)
- 🔧 Simple and extensible: plug into any API

---
