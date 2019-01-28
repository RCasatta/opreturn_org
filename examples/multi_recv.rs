use std::sync::mpsc::sync_channel;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;

fn main() {
    let (send,receive) = sync_channel(2);
    let rec = Arc::new(Mutex::new(receive));

    let mut handles = vec![];
    let thread = 100;
    for i in 0..thread {
        let rec_clone = rec.clone();
        let handle = thread::spawn(move || {
            let result = rec_clone.lock().unwrap().recv();
            thread::sleep_ms(5000);
            println!("t{} result:{} ", i, result.unwrap());
        });
        handles.push(handle);
    }

    for i in 0..thread {
        send.send(0).unwrap();
        thread::sleep_ms(50);
    }

    loop {
        match handles.pop() {
            Some(handle) => handle.join().unwrap(),
            None => break,
        }
    }

}