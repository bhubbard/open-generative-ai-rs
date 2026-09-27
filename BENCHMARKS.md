# Benchmark Report: `open-generative-ai-rs` (Rust) vs. Python Generative Pipelines

*Conducted on Apple Silicon comparing native Rust `open-generative-ai-rs` against Python GenAI.*

---

## 1. Noise Scheduling & Conditioning Throughput

| Operation | `open-generative-ai-rs` | Python Pipeline | Speedup Factor |
| :--- | :---: | :---: | :---: |
| **Noise Schedule Pre-computation (1,000 steps)** | **14 µs** | 850 µs | **60.7× faster** |
| **Conditioning Vector Scaling** | **8 µs** | 320 µs | **40.0× faster** |
