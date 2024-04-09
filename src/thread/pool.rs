use std::sync::{Arc, mpsc, Mutex};
use std::thread;

enum Message {
    NewJob(Job),
    Terminate
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker{
    id:usize,
    thread:Option<thread::JoinHandle<()>>
}

impl Worker{
    fn new(id:usize,receiver:Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker{
        let thread = thread::spawn(move || loop {
           match receiver.lock().unwrap().recv().unwrap() {
               Message::NewJob(job) => {
                   println!("worker {} got a job; executing.",id);
                   job()
               }
               Message::Terminate => {
                   println!("worker {} was told to terminate.",id);
                   break;
               }
           }
        });
        Worker{
            id,
            thread:Some(thread)
        }
    }
}

pub struct ThreadPool{
    workers:Vec<Worker>,
    sender:mpsc::Sender<Message>
}

impl ThreadPool{
    pub fn new(n_threads:usize) -> ThreadPool{
        assert!(n_threads > 0);
        let (sender,receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(n_threads);
        for id in 0..n_threads{
            workers.push(Worker::new(id,Arc::clone(&receiver)));
        }
        ThreadPool{
            workers,
            sender
        }
    }

    pub fn execute<F>(&self,f:F) where F:FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("sending terminate message to all workers.");
        for _ in &self.workers{
            self.sender.send(Message::Terminate).unwrap()
        }
        println!("shutting down all workers.");
        for worker in &mut self.workers{
            println!("Shutting down worker {}",worker.id);
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }
        }
    }
}