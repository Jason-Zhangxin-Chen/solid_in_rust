// Cargo.toml dependencies:
// [dependencies]
// crossbeam = "0.8"
// crossbeam-channel = "0.5"

use crossbeam_channel::{bounded, unbounded, select};
use std::thread;
use std::time::Duration;

// ============================================================
// 1. MPSC — Multi-Producer, Single-Consumer
//    Many threads send work; one thread collects the results.
// ============================================================
fn mpsc_example() {
    println!("\n=== MPSC: Multi-Producer, Single-Consumer ===");

    let (tx, rx) = unbounded::<String>();
    let num_producers = 4;

    // Spawn multiple producers
    let handles: Vec<_> = (0..num_producers)
        .map(|id| {
            let tx = tx.clone(); // clone Sender for each producer
            thread::spawn(move || {
                for i in 0..3 {
                    let msg = format!("Producer-{id} → message {i}");
                    println!("  [SEND] {msg}");
                    tx.send(msg).unwrap();
                    thread::sleep(Duration::from_millis(50));
                }
                // tx drops here when the thread finishes
            })
        })
        .collect();

    // Drop the original tx so the channel closes when all producers finish
    drop(tx);

    // Single consumer drains all messages
    thread::spawn(move || {
        for msg in &rx {
            println!("  [RECV] Consumer got: {msg}");
        }
        println!("  [RECV] Channel closed, consumer done.");
    });

    for h in handles {
        h.join().unwrap();
    }

    thread::sleep(Duration::from_millis(300)); // let consumer finish
}

// ============================================================
// 2. SPSC — Single-Producer, Single-Consumer
//    One sender, one receiver — simplest, lowest overhead.
//    crossbeam-channel optimizes this path automatically.
// ============================================================
fn spsc_example() {
    println!("\n=== SPSC: Single-Producer, Single-Consumer ===");

    // bounded(N) adds backpressure: producer blocks when buffer is full
    let (tx, rx) = bounded::<u64>(8);

    let producer = thread::spawn(move || {
        for i in 0u64..6 {
            println!("  [SEND] Producing item {i}");
            tx.send(i).unwrap(); // blocks if buffer is full
            thread::sleep(Duration::from_millis(40));
        }
        // tx drops → channel closes → consumer's loop ends
    });

    let consumer = thread::spawn(move || {
        for item in &rx {
            println!("  [RECV] Consumed item {item}");
            thread::sleep(Duration::from_millis(80)); // consumer is slower
        }
        println!("  [RECV] Consumer finished.");
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

// ============================================================
// 3. SPMC — Single-Producer, Multi-Consumer (Fan-out / Work-stealing)
//    One sender distributes tasks; many workers compete for them.
//    crossbeam's rx is Clone, so multiple threads can receive.
// ============================================================
fn spmc_example() {
    println!("\n=== SPMC: Single-Producer, Multi-Consumer ===");

    let (tx, rx) = bounded::<u32>(16);
    let num_workers = 3;

    // Single producer sends all tasks
    let producer = thread::spawn(move || {
        for task_id in 0..9u32 {
            println!("  [SEND] Enqueuing task {task_id}");
            tx.send(task_id).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        // tx drops → workers' for loops end
    });

    // Multiple consumers, each cloning rx — they compete (work-steal) for tasks
    let workers: Vec<_> = (0..num_workers)
        .map(|worker_id| {
            let rx = rx.clone(); // clone Receiver for each worker
            thread::spawn(move || {
                for task in &rx {
                    println!("  [RECV] Worker-{worker_id} processing task {task}");
                    thread::sleep(Duration::from_millis(60)); // simulate work
                }
            })
        })
        .collect();

    producer.join().unwrap();
    for w in workers {
        w.join().unwrap();
    }
}

// ============================================================
// 4. MPMC — Multi-Producer, Multi-Consumer
//    The most flexible pattern: N senders, M receivers.
//    Great for thread pools where any thread can produce or consume.
// ============================================================
fn mpmc_example() {
    println!("\n=== MPMC: Multi-Producer, Multi-Consumer ===");

    let (tx, rx) = bounded::<String>(32);
    let num_producers = 2;
    let num_consumers = 3;

    // Multiple producers
    let producers: Vec<_> = (0..num_producers)
        .map(|pid| {
            let tx = tx.clone();
            thread::spawn(move || {
                for i in 0..4 {
                    let msg = format!("P{pid}:item{i}");
                    println!("  [SEND] {msg}");
                    tx.send(msg).unwrap();
                    thread::sleep(Duration::from_millis(30));
                }
            })
        })
        .collect();

    drop(tx); // drop original so channel closes when all producers finish

    // Multiple consumers
    let consumers: Vec<_> = (0..num_consumers)
        .map(|cid| {
            let rx = rx.clone();
            thread::spawn(move || {
                for msg in &rx {
                    println!("  [RECV] Consumer-{cid} ← {msg}");
                    thread::sleep(Duration::from_millis(50));
                }
            })
        })
        .collect();

    for p in producers {
        p.join().unwrap();
    }
    for c in consumers {
        c.join().unwrap();
    }
}

// ============================================================
// BONUS: select! — wait on multiple channels simultaneously
//    Useful when a thread needs to listen on several channels
//    at once (e.g., a command channel + a data channel).
// ============================================================
fn select_example() {
    println!("\n=== BONUS: select! across multiple channels ===");

    let (data_tx, data_rx) = bounded::<u32>(8);
    let (stop_tx, stop_rx) = bounded::<()>(1);

    // Producer: sends data, then signals stop
    thread::spawn(move || {
        for i in 0..5u32 {
            data_tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(40));
        }
        stop_tx.send(()).unwrap();
    });

    // Consumer: reacts to whichever channel fires first
    loop {
        select! {
            recv(data_rx) -> msg => {
                match msg {
                    Ok(val) => println!("  [DATA] received {val}"),
                    Err(_)  => println!("  [DATA] channel closed"),
                }
            },
            recv(stop_rx) -> _ => {
                println!("  [STOP] received stop signal, exiting.");
                break;
            },
        }
    }
}

fn main() {
    mpsc_example();
    spsc_example();
    spmc_example();
    mpmc_example();
    select_example();

    println!("\nAll examples complete.");
}
