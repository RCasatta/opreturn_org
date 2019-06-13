use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;

pub struct Read {
    path: PathBuf,
    sender: SyncSender<Option<Vec<u8>>>,
}

impl Read {
    pub fn new(path: PathBuf, sender: SyncSender<Option<Vec<u8>>>) -> Self {
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
        for path in paths.iter() {
            let blob = fs::read(path).unwrap_or_else(|_| panic!("failed to read {:?}", path));
            let len = blob.len();
            println!("read {} of {:?}", len, path);
            self.sender.send(Some(blob)).expect("cannot send");
        }
        self.sender.send(None).expect("cannot send");
        println!("ending  reader");
    }
}
