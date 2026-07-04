use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // --- CONFIGURATION ---
    let target_server = "127.0.0.1:8080";
    let concurrency = 50;
    let test_duration = Duration::from_secs(60);
    // ---------------------

    println!("⚡ Starting Rust TCP Cache bench on {}...", target_server);
    println!("👥 Concurrency: {} | ⏱️ Duration: {:?}", concurrency, test_duration);

    let ops_set = Arc::new(AtomicU64::new(0));
    let ops_get = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));
    
    let running = Arc::new(AtomicU64::new(1)); // 1 = running, 0 = stop
    let mut handles = vec![];

    let start_time = Instant::now();

    // Spawn concurrent worker threads
    for worker_id in 0..concurrency {
        let ops_set = Arc::clone(&ops_set);
        let ops_get = Arc::clone(&ops_get);
        let errors = Arc::clone(&errors);
        let running = Arc::clone(&running);
        let target = target_server.to_string();

        let handle = thread::spawn(move || {
            // Open a raw persistent synchronous TCP stream
            let stream = match TcpStream::connect(&target) {
                Ok(s) => s,
                Err(_) => {
                    errors.fetch_add(1, Ordering::Relaxed);
                    return;
                }
            };
            
            // Disable Nagle's algorithm for minimal latency
            let _ = stream.set_nodelay(true);
            
            let mut writer = stream;
            // Clone the stream handle for buffered reading
            let mut reader = BufReader::new(writer.try_clone().unwrap());
            let mut response_buffer = String::new();

            // Simple thread-local pseudo-random counter for unique keys
            let mut key_counter = worker_id * 100_000;

            while running.load(Ordering::Relaxed) == 1 {
                key_counter += 1;
                let key = format!("key_{}", key_counter);
                let val = "cached_payload_data";
                let ttl_ms = 5000;

                // 1. EXECUTE SET ("SET key value TTL\n")
                let set_cmd = format!("SET {} {} {}\n", key, val, ttl_ms);
                if writer.write_all(set_cmd.as_bytes()).is_err() {
                    errors.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                
                response_buffer.clear();
                if reader.read_line(&mut response_buffer).is_err() {
                    errors.fetch_add(1, Ordering::Relaxed);
                    return; // Terminate thread if connection breaks
                }
                ops_set.fetch_add(1, Ordering::Relaxed);

                // 2. EXECUTE GET ("GET key\n")
                let get_cmd = format!("GET {}\n", key);
                if writer.write_all(get_cmd.as_bytes()).is_err() {
                    errors.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                response_buffer.clear();
                if reader.read_line(&mut response_buffer).is_err() {
                    errors.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                ops_get.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    // Let the threads hammer the cache server for the designated time
    thread::sleep(test_duration);
    running.store(0, Ordering::Relaxed); // Signal threads to stop

    // Wait for all worker threads to wind down cleanly
    for handle in handles {
        let _ = handle.join();
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let total_set = ops_set.load(Ordering::Relaxed);
    let total_get = ops_get.load(Ordering::Relaxed);
    let total_err = errors.load(Ordering::Relaxed);
    let total_ops = total_set + total_get;

    println!("\n--- 📊 Performance Summary ---");
    println!("Elapsed Time:   {:.2}s", elapsed);
    println!("Successful SET: {} ({:.2} ops/sec)", total_set, total_set as f64 / elapsed);
    println!("Successful GET: {} ({:.2} ops/sec)", total_get, total_get as f64 / elapsed);
    println!("Total Rate:     {:.2} combined ops/sec", total_ops as f64 / elapsed);
    if total_err > 0 {
        println!("⚠️ Total Errors: {}", total_err);
    } else {
        println!("🎉 Total Errors: 0 (Clean Run)");
    }
}