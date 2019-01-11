use crate::parse::BlockParsed;
use crate::{Startable};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use bitcoin::util::hash::BitcoinHash;
use std::time::Instant;
use std::time::Duration;
use plotlib::style::Point;
use plotlib::scatter::Scatter;
use plotlib::view::View;

pub struct Blocks {
    sender : Sender<Option<BlockParsed>>,
    receiver : Receiver<Option<BlockParsed>>,
}

impl Blocks {
    pub fn new() -> Blocks {
        let (sender, receiver) = channel();
        Blocks {
            sender,
            receiver,
        }
    }

    pub fn get_sender(&self) -> Sender<Option<BlockParsed>> {
        self.sender.clone()
    }
}

impl Startable for Blocks {
    fn start(&self) {
        println!("starting blocks processer");
        let mut sum_size = 0u64;
        let mut max_size = 0;
        let mut min_hash = "Z".to_string();
        let mut wait_time = Duration::from_secs(0);
        let mut nonce_points = vec![];

        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("can't receive in blocks");
            wait_time += instant.elapsed();
            match received {
                Some(block) => {
                    let header = block.block_header;
                    let size = block.size;
                    let height = block.height;

                    let cur_hash = header.bitcoin_hash().to_string();
                    if min_hash > cur_hash {
                        min_hash = cur_hash;
                        println!("min hash: {} at height {}", min_hash, height );
                    }
                    if max_size < size {
                        max_size = size;
                        println!("max size: {} at height {}", max_size, height );
                    }
                    if block.height % 20000 == 0 {
                        println!("height: {} ", height);
                    }
                    sum_size += size as u64;
                    nonce_points.push((height as f64,block.block_header.nonce as f64));
                },
                None => break,
            }
        }

        let s1 = Scatter::from_vec(&nonce_points).style(
            plotlib::scatter::Style::new()
                .marker(plotlib::style::Marker::Square)
                .colour("#DD3355")
                .size(2.),
        );

        let v = View::new()
            .add(&s1)
            .x_label("Some varying variable")
            .y_label("The response of something");
        plotlib::page::Page::single(&v).save("scatter.svg");

        println!("sum = {}", sum_size);
        println!("ending blocks processer, wait time: {:?}", wait_time );
    }
}
