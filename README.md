# Beet: High-Performance TCP Cache Server

Beet is an ultra-fast, zero-allocation TCP cache server written in Rust, leveraging the modern edition2024 toolchain. It provides microsecond-level key-value caching utilizing an explicit frame protocol for processing data lifecycle streams using commands like SET and GET alongside precise millisecond-level TTL configuration.

---

## Key Features
* Zero-Allocation Protocol Parsing: Optimized for high-throughput stream ingestion.
* Millisecond-Level TTL: Fine-grained eviction mechanics to clear memory frames deterministically.
* Multi-Stage Minimal Docker Footprint: Compiled down to an isolated runtime layer using a tiny, hardened Debian container.
* Isolated Multi-Threaded Load Tester: Embedded native benchmarking mini-binary designed to evaluate high concurrency under load.

---

## Docker Integration and Setup

Because running benchmarks locally can mask concurrency issues (due to the client and server competing for identical CPU cores), the application should be deployed inside an isolated Docker container with strict resource limits.

### 1. Build the Docker Image
Ensure your local .env file is present in your project root before building. The Docker builder utilizes the modern Rust toolchain to compile release dependencies:
```bash
docker build -t beet-tcp-server .
```

### 2. Run the Container with Resource Limits
To simulate production workloads and isolate performance boundaries on a powerful development machine, execute the container with constrained constraints:
```bash
docker run -d \
  -p 8080:8080 \
  --name beet \
  --cpus="2.0" \
  --memory="512m" \
  beet-tcp-server
```
*Note: Ensure your server configuration maps the binding interface to `0.0.0.0:8080` instead of `127.0.0.1:8080` to allow Docker's network bridge to route your host's frames into the container.*

### 3. Diagnostics and Verifying Health
```bash
# Check running containers
docker ps

# Inspect runtime connection logs
docker logs beet

# Sanity check a manual operation using netcat (BSD variant)
printf "GET test_key\n" | nc -N 127.0.0.1 8080
```

---

## Running the Performance Load Test

The repository includes an isolated Cargo mini-binary benchmark workflow (`src/bin/cache_bench.rs`). This script avoids heavy testing frameworks and relies completely on safe, atomic standard primitives—maintaining full pipeline pressure without resource leakage.

### The Network Frame Protocol Rules
* **SET Command:** `SET <key> <value> <TTL_in_milliseconds>\n`
* **GET Command:** `GET <key>\n`

### Run the Benchmark Script
Navigate to your main project directory on your host machine and execute the following command. The `--release` flag is vital to avoid client-side CPU bottlenecks:
```bash
cargo run --release --bin cache_bench
```