use crate::process::{ProcessBip158Stats, ProcessOpRet, ProcessStats, ProcessTxStats};
use blocks_iterator::log::info;
use blocks_iterator::{PeriodCounter, PipeIterator};
use chrono::format::StrftimeItems;
use chrono::Utc;
use clap::Parser;
use env_logger::Env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fs, io, thread};

mod charts;
mod counter;
mod pages;
mod process;

#[derive(Parser, Debug, Clone)]
struct Params {
    /// Where to produce the website
    #[clap(short, long)]
    pub target_dir: PathBuf,

    /// Parse pubkeys (which is expensive involving EC cryptography)
    #[clap(short, long)]
    pub parse_pubkeys: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Instant::now();
    init_logging();
    info!("start");

    let params = Params::parse();
    if !fs::metadata(&params.target_dir).unwrap().is_dir() {
        panic!("--target-dir must be a directory");
    }
    let mut site_home = params.target_dir.clone();
    site_home.push("site");
    if !site_home.exists() {
        fs::create_dir_all(&site_home).unwrap();
    }
    let mut raw_home = params.target_dir.clone();
    raw_home.push("raw");
    if !raw_home.exists() {
        fs::create_dir_all(&raw_home).unwrap();
    }

    let blocks_size = 3;

    let iter = PipeIterator::new(io::stdin(), None);

    let (send_1, receive_1) = sync_channel(blocks_size);
    let (send_2, receive_2) = sync_channel(blocks_size);
    let (send_3, receive_3) = sync_channel(blocks_size);
    let (send_4, receive_4) = sync_channel(blocks_size);
    let senders = [send_1, send_2, send_3, send_4];

    let process = ProcessOpRet::new(receive_1, &params.target_dir, params.parse_pubkeys);
    let process_handle = thread::spawn(move || process.start());

    let process_stats = ProcessStats::new(receive_2, &params.target_dir);
    let process_stats_handle = thread::spawn(move || process_stats.start());

    let process_bip158 = ProcessBip158Stats::new(receive_3, &params.target_dir);
    let process_bip158_handle = thread::spawn(move || process_bip158.start());

    let process_tx_stats = ProcessTxStats::new(receive_4, &params.target_dir);
    let process_tx_stats_handle = thread::spawn(move || process_tx_stats.start());

    let mut period = PeriodCounter::new(Duration::from_secs(10));

    for block_extra in iter {
        if period.period_elapsed().is_some() {
            info!(
                "# {:7} {} {:?}",
                block_extra.height(),
                block_extra.block_hash(),
                block_extra.fee()
            );
        }
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
        let mut page_path = site_home.clone();
        page_path.push(&page.permalink);
        if !page_path.exists() {
            fs::create_dir_all(&page_path).unwrap();
        }
        page_path.push("index.html");
        fs::write(page_path, page_html).unwrap();
    }
    let mut index_path = site_home.clone();
    index_path.push("index.html");
    let index = pages::create_index(&pages);
    fs::write(index_path, index.into_string()).unwrap();

    let mut about_path = site_home.clone();
    about_path.push("about");
    if !about_path.exists() {
        fs::create_dir_all(&about_path).unwrap();
    }
    about_path.push("index.html");
    let about = pages::create_about();
    fs::write(about_path, about.into_string()).unwrap();

    // favicon
    let mut favicon_file = site_home.clone();
    favicon_file.push("favicon.ico");
    let favicon = include_bytes!("../target_dir/site/favicon.ico");
    fs::write(favicon_file, &favicon).unwrap();

    info!("end, elapsed:{}s", now.elapsed().as_secs());
    Ok(())
}

fn now() -> String {
    let now = Utc::now().naive_utc();
    let fmt = StrftimeItems::new("%Y-%m-%d %H:%M:%S");
    format!("{}", now.format_with_items(fmt))
}

fn init_logging() {
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    if let Ok(s) = std::env::var("RUST_LOG_STYLE") {
        if s == "SYSTEMD" {
            builder.format(|buf, record| {
                let level = match record.level() {
                    log::Level::Error => 3,
                    log::Level::Warn => 4,
                    log::Level::Info => 6,
                    log::Level::Debug => 7,
                    log::Level::Trace => 7,
                };
                writeln!(buf, "<{}>{}: {}", level, record.target(), record.args())
            });
        }
    }

    builder.init();
}
