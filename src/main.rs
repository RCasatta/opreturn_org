use crate::process::{ProcessBip158Stats, ProcessOpRet, ProcessStats, ProcessTxStats};
use blocks_iterator::log::{info, log};
use blocks_iterator::structopt::StructOpt;
use blocks_iterator::{periodic_log_level, PipeIterator};
use chrono::format::StrftimeItems;
use chrono::Utc;
use env_logger::Env;
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::{fs, io, thread};

mod charts;
mod pages;
mod process;
mod templates;

#[derive(StructOpt, Debug, Clone)]
struct Params {
    /// Where to produce the website
    #[structopt(short, long)]
    pub target_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");

    let params = Params::from_args();
    if !fs::metadata(&params.target_dir).unwrap().is_dir() {
        panic!("--target-dir must be a directory");
    }
    if !params.target_dir.exists() {
        fs::create_dir_all(&params.target_dir).unwrap();
    }

    let blocks_size = 3;

    let iter = PipeIterator::new(io::stdin(), io::stdout());

    let (send_1, receive_1) = sync_channel(blocks_size);
    let (send_2, receive_2) = sync_channel(blocks_size);
    let (send_3, receive_3) = sync_channel(blocks_size);
    let (send_4, receive_4) = sync_channel(blocks_size);
    let senders = [send_1, send_2, send_3, send_4];

    let process = ProcessOpRet::new(receive_1);
    let process_handle = thread::spawn(move || process.start());

    let process_stats = ProcessStats::new(receive_2, &params.target_dir);
    let process_stats_handle = thread::spawn(move || process_stats.start());

    let process_bip158 = ProcessBip158Stats::new(receive_3, &params.target_dir);
    let process_bip158_handle = thread::spawn(move || process_bip158.start());

    let process_tx_stats = ProcessTxStats::new(receive_4);
    let process_tx_stats_handle = thread::spawn(move || process_tx_stats.start());

    for block_extra in iter {
        log!(
            periodic_log_level(block_extra.height, 10_000),
            "# {:7} {} {:?}",
            block_extra.height,
            block_extra.block_hash,
            block_extra.fee()
        );
        let block_extra = Arc::new(Some(block_extra));
        for sender in senders.iter() {
            sender.send(block_extra.clone()).unwrap();
        }
    }
    let end = Arc::new(None);
    for sender in senders.iter() {
        sender.send(end.clone()).unwrap();
    }

    let bip158_stats = process_bip158_handle.join().expect("couldn't join");
    let process_stats = process_stats_handle.join().expect("couldn't join");
    let (opret, script_type) = process_handle.join().expect("couldn't join");
    let tx_stats = process_tx_stats_handle.join().expect("couldn't join");

    let pages = pages::get_pages(
        &bip158_stats,
        &opret,
        &script_type,
        &process_stats,
        &tx_stats,
    );
    for page in pages.iter() {
        let page_html = page.to_html().into_string();
        let filestring = format!(
            "{}/{}/index.html",
            params.target_dir.display(),
            page.permalink
        );
        let file: PathBuf = filestring.into();
        let parent_dir = file.parent().unwrap();
        if !parent_dir.exists() {
            fs::create_dir_all(&parent_dir).unwrap();
        }
        fs::write(file, page_html).unwrap();
    }
    let indexstring = format!("{}/index.html", params.target_dir.display(),);
    let index = pages::create_index(&pages);
    fs::write(indexstring, index.into_string()).unwrap();

    let contact_string = format!("{}/contact/index.html", params.target_dir.display(),);
    let contact_file: PathBuf = contact_string.into();
    let parent_dir = contact_file.parent().unwrap();
    if !parent_dir.exists() {
        fs::create_dir_all(&parent_dir).unwrap();
    }
    let contact = pages::create_contact();
    fs::write(contact_file, contact.into_string()).unwrap();

    info!("end");
    Ok(())
}

fn now() -> String {
    let now = Utc::now().naive_utc();
    let fmt = StrftimeItems::new("%Y-%m-%d %H:%M:%S");
    format!("{}", now.format_with_items(fmt))
}
