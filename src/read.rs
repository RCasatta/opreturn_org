use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;
use std::time::Instant;

pub struct Read {
    path: PathBuf,
    sender: Vec<SyncSender<Option<Vec<u8>>>>,
}

impl Read {
    pub fn new(path: PathBuf, sender: Vec<SyncSender<Option<Vec<u8>>>>) -> Self {
        Read { path, sender }
    }

    pub fn start(&mut self) {
        self.path.push("blocks");
        self.path.push("blk*.dat");
        println!("listing block files at {:?}", self.path);
        let mut paths: Vec<PathBuf> = glob::glob(self.path.to_str().unwrap())
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        paths.sort();
        println!("There are {} block files", paths.len());
        let mut busy_time = 0u128;
        let mut count = 0usize;
        for path in paths.iter() {
            let now = Instant::now();
            let blob = fs::read(path).unwrap_or_else(|_| panic!("failed to read {:?}", path));
            let len = blob.len();
            println!("read {} of {:?}", len, path);
            busy_time = busy_time + now.elapsed().as_nanos();
            self.sender[count % 2]
                .send(Some(blob))
                .expect("cannot send");
            count += 1;
        }
        self.sender[0].send(None).expect("cannot send");
        self.sender[1].send(None).expect("cannot send");
        println!("ending reader, busy time: {}s", (busy_time / 1_000_000_000));
    }
}
