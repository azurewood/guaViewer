# China Stock Market Viewer

 > **Version:** v0.5.0
 
 > **Status:** Experimental but usable
 
 > **Language:** Rust
 
 > **Transport:** TCP (TDX-compatible protocol)

---

## Overview

This project is a **lightweight TCP-based market viewing tool** built on top of a custom `FeedClient` derived from the original **Rustdx TCP implementation**.

While the upstream Rustdx project moved away from TCP in favor of a commercial Web API starting from v0.4, this project continues to explore and validate the feasibility of **direct TCP market data access**, inspired by the long-standing Pytdx ecosystem.

The tool is already **functional for real-world use** and has not exhibited fundamental transport or protocol-level issues so far.

---

## Background

Starting from **Rustdx v0.4**, the original author discontinued the TCP transport layer and migrated to a commercial Web API.
The main reason cited was **IP instability** when connecting to upstream quote servers.

The original plan was to implement a Pytdx-like strategy, including:

* IP probing and health checks
* Connection pooling
* Automatic failover

However, due to the operational complexity and long-term maintenance cost, this effort was eventually abandoned.

Despite this shift, the earlier Rustdx codebase laid down a **very solid technical foundation**, including:

* Well-defined binary protocol parsing
* Clean data structures
* Clear separation between transport, decoding, and domain logic

This project builds upon that foundation and reintroduces a **carefully scoped TCP client**, focusing on correctness, observability, and simplicity.

---

## Design Rationale

### Why TCP Instead of a Web API?

Choosing TCP over a Web API is a **deliberate architectural decision**, not an attempt to compete with commercial data providers.

#### 1. Transparency & Control

* TCP exposes the **raw protocol and data flow**
* No hidden throttling, opaque retry logic, or undocumented limits
* Easier to reason about latency, failures, and retries

#### 2. Architectural Simplicity

* No HTTP stacks, JSON parsing, or async frameworks required
* Fewer dependencies
* Easier to debug with packet inspection and logs

#### 3. Pytdx-Proven Model

* Pytdx has demonstrated that **TCP + IP pool + retry logic works**
* Problems are operational, not theoretical
* This project intentionally scopes the problem instead of overgeneralizing

#### 4. Vendor Independence

* No API keys
* No usage quotas
* No commercial lock-in
* Suitable for research, offline analysis, and private deployments

#### 5. Explicit Failure Handling

* TCP failures are visible and explicit
* Encourages correct handling of timeouts, reconnects, and backpressure
* Avoids “silent degradation” common in wrapped Web APIs

> **Trade-off acknowledged:** TCP requires more engineering discipline.
> This project embraces that cost in exchange for clarity and control.

---

## Project Comparison

| Feature / Project | Pytdx    | Rustdx (≥0.4)        | This Project       |
| ----------------- | -------- | -------------------- | ------------------ |
| Transport         | TCP      | Web API (Commercial) | TCP                |
| IP Pooling        | ✅ Mature | ❌                    | ✅     |
| Failover          | ✅        | ❌                    | ✅     |
| Connection Pool   | ✅        | ❌                    | ✅                  |
| Blocking API      | ✅        | ❌                    | ✅                  |
| Async Dependency  | ❌        | ✅ (HTTP stack)       | ⚠️ (by design)      |
| Vendor Lock-in    | ❌        | ✅                    | ❌                  |
| Debuggability     | High     | Medium               | High               |
| Production Scope  | Broad    | Broad                | Focused / Explicit |

Legend:

* ✅ Implemented
* ⚠️ Partial / evolving
* ❌ Not present

---

## Current Status (v0.5.0)

**What works today:**

* TCP-based `FeedClient`
* Stable request / response decoding
* K-line (candlestick) data fetching
* Thread-based concurrency (no Tokio)
* Usable market viewer UI
* Explicit error propagation
* Deterministic blocking behavior

**What this version is *not*:**

* A drop-in Rust replacement for Pytdx
* A fully automated, self-healing data service
* A commercial-grade market data platform

This release is intentionally **conservative in scope**.

---

## Roadmap (along with tcpTDX library)

### v0.3.x

* IP health scoring
* Smarter retry strategy
* Configurable timeout policies
* Improved logging & metrics

### v0.4.x

* Batch request optimizations
* Better backpressure handling

## v0.5.x

* Android and Windows support

### Future (Non-Goals)

* Replacing commercial Web APIs
* Chasing maximum throughput at all costs
* Abstracting away TCP semantics
* Optional persistence layer

---

## Philosophy

This project values:

* **Correctness over cleverness**
* **Explicit behavior over hidden automation**
* **Understandability over abstraction**

If you are comfortable with TCP, blocking I/O, and explicit failure handling, this tool aims to give you **full visibility and control** over your market data pipeline.

---

## License

MIT
