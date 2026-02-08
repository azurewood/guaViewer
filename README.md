<div align="center">
  <img width="96" src="./guaviewer/assets/icons/icon_.png" alt="Project Icon">
</div>

<h1 align="center">
  China Stock Market Viewer
</h1>

<p align="center">
  A lightweight TCP-based desktop market viewer built with Rust
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-v0.5.0-orange.svg">
  <img src="https://img.shields.io/badge/language-Rust-orange.svg">
  <img src="https://img.shields.io/badge/transport-TCP-blue.svg">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgreen.svg">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg">
</p>

<p align="center">
  <a href="https://slint.dev">
    <img alt="Made with Slint" src="https://raw.githubusercontent.com/slint-ui/slint/master/logo/MadeWithSlint-logo-light.svg" height="56">
  </a>
</p>

<p align="center">
  English | <a href="./README_zh.md">简体中文</a>
</p>

---

## :pushpin: Brief

**China Stock Market Viewer** is a lightweight, experimental-but-usable desktop application for viewing Chinese stock market data over a **direct TCP connection** using a **TDX-compatible protocol**.

The project is built on top of a custom `FeedClient` derived from the original **Rustdx TCP implementation**, and is designed to validate whether **direct TCP market data access** remains viable, transparent, and controllable compared to modern commercial Web APIs.

Unlike upstream Rustdx (≥ v0.4), which migrated to a commercial HTTP API, this project intentionally continues to explore the **Pytdx-style TCP model**, emphasizing correctness, observability, and explicit failure handling.

---

## :mag: Preview

<div align="center">
  <img src="guaviewer/assets/images/preview.png" alt="Application Preview">
</div>

---

## Overview

This project provides:

- A **blocking, deterministic TCP client**
- Explicit request/response decoding
- A usable **desktop market viewer UI**
- Thread-based concurrency without Tokio
- Clear error propagation and failure visibility

The tool is already functional for **real-world exploratory and research use**, and no fundamental protocol-level issues have been observed so far.

---

## Background

Starting from **Rustdx v0.4**, the original project discontinued its TCP transport layer and adopted a commercial Web API, primarily due to **IP instability and operational complexity**.

The original TCP roadmap included:

- IP probing and health checks  
- Connection pooling  
- Automatic failover  

However, these features were ultimately abandoned upstream due to long-term maintenance costs.

Despite this, the early Rustdx TCP implementation established a **strong technical foundation**, including:

- Well-structured binary protocol parsing  
- Clean domain data models  
- Clear separation of transport and decoding logic  

This project builds upon that foundation and reintroduces a **carefully scoped TCP client**, deliberately limiting scope to maintain clarity and robustness.

---

## Design Rationale

### Why TCP Instead of a Web API?

This is a **deliberate architectural choice**, not an attempt to replace commercial data providers.

**Key motivations:**

1. **Transparency & Control**  
   - Raw protocol visibility  
   - No hidden throttling or opaque retries  

2. **Architectural Simplicity**  
   - No HTTP stacks or JSON parsing  
   - Fewer dependencies, easier debugging  

3. **Pytdx-Proven Model**  
   - TCP + IP pool + retry works in practice  
   - Operational problems are manageable when scoped  

4. **Vendor Independence**  
   - No API keys or quotas  
   - Suitable for private or offline use  

5. **Explicit Failure Handling**  
   - Timeouts, reconnects, and backpressure are visible  
   - Avoids silent degradation common in wrapped APIs  

> Trade-off acknowledged: TCP demands stricter engineering discipline.  
> This project embraces that cost in exchange for control and clarity.

---

## Project Comparison

| Feature / Project | Pytdx | Rustdx (≥0.4) | This Project |
|------------------|-------|---------------|--------------|
| Transport        | TCP   | Web API       | TCP          |
| IP Pooling       | ✅    | ❌            | ✅           |
| Failover         | ✅    | ❌            | ✅           |
| Connection Pool  | ✅    | ❌            | ✅           |
| Blocking API     | ✅    | ❌            | ✅           |
| Async Stack      | ❌    | ✅            | ⚠️ (by design) |
| Data Pipeline    | ❌    | ✅            | ✅ (built-in) |
| Vendor Lock-in   | ❌    | ✅            | ❌           |
| Debuggability    | High  | Medium        | High         |

Legend:  
✅ Implemented ⚠️ Partial ❌ Not present

---

## Current Status (v0.5.0)

**What works:**

- TCP-based `FeedClient`
- Stable protocol decoding
- K-line (candlestick) data fetching
- Thread-based concurrency
- Desktop UI viewer
- Deterministic blocking behavior

**What this is not:**

- A drop-in Pytdx replacement  
- A self-healing production data service  
- A commercial market data platform  

The scope is intentionally **conservative and explicit**.

---

## Roadmap

### v0.3.x
- IP health scoring  
- Smarter retry strategies  
- Configurable timeout policies  
- Improved logging and metrics  

### v0.4.x
- Batch request optimizations  
- Improved backpressure handling  

### v0.5.x
- Android support  
- Windows distribution improvements  

### Non-Goals
- Replacing commercial APIs  
- Maximizing throughput at all costs  
- Abstracting away TCP semantics  

---

## Philosophy

This project prioritizes:

- **Correctness over cleverness**
- **Explicit behavior over hidden automation**
- **Understandability over abstraction**

If you are comfortable with TCP, blocking I/O, and visible failure modes, this tool aims to give you **full control over your market data pipeline**.

---

## License

MIT
