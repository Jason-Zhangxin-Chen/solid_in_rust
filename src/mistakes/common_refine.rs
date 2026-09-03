use std::sync::{mpsc, Arc, Mutex};
use std::thread;
// Error when calling method that take ownership on a reference, this is a common mistake when
// you want to call a method that takes ownership / consumes data of self on a reference.
use std::thread::JoinHandle;
struct Worker{
    thread: JoinHandle<()>,
    id: usize,
}

struct ThreadPool {
    workers: Vec<Worker>,
}

// The drop takes mut reference, however it consumes thread value when calling .join()
impl Drop for ThreadPool {
    fn drop(&mut self) {

        for worker in &mut self.workers {
            worker.thread.join().unwrap(); // ❌ Error: cannot move out of `worker.thread`
        }

        //[option-1] an alternative solution is to drain the vector.
        Vec::drain(&mut self.workers, .. ).for_each(|worker| {
            println!("drop worker thread {}", worker.id);
            worker.thread.join().unwrap();
        });
    }
}

// better implementation with a cheap Option<T> wrapper.
struct ThreadPoolV2{
    workers: Vec<Option<Worker>>,
}

impl Drop for ThreadPoolV2 {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(worker) = worker.take() {
                println!("drop worker thread {}", worker.id);
                worker.thread.join().unwrap();
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

// Subtle case using Mutex<T>.
// a classic trap around temporary lifetimes with while let (and if let, match) vs. plain let.
// It directly affects how long a Mutex remains locked.
// subtle case
impl Worker {
    fn new_wrong_version(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            // while let (and if let and match) does not drop temporary values until the end of
            // the associated block the mutex will not be unlocked until associated block ends
            while let Ok(job) = receiver.lock().unwrap().recv() {
                println!("Worker {} got a job; executing.", id);

                job();
            }// !!! the mutex is unlocked here after the loop is finished.
        });

        Worker { id, thread }
    }
}

// correct case
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // with let, any temporary values used in the expression on the right hand side
            // of the equals sign are immediately dropped when the let statement ends.
            // It means, the lock is unlocked right after the statement ends.
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {} got a job; executing.", id);

            job();
        });

        Worker { id, thread }
    }
}

