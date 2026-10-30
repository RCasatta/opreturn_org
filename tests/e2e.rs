//! Runs the binary on `tests/fixtures/testnet.bin` and compares the generated site with the
//! snapshot in `tests/snapshots/site`. Run with `UPDATE_SNAPSHOTS=1` to accept the new output.

use blocks_iterator::bitcoin::consensus::Decodable;
use blocks_iterator::BlockExtra;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/testnet.bin");
const SNAPSHOT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/site");

/// Runs the binary on the fixture writing output in a fresh dir named `name`
fn run(name: &str, extra_args: &[&str]) -> PathBuf {
    let target_dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir).unwrap();
    }
    fs::create_dir_all(&target_dir).unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_opreturn_org"))
        .arg("--target-dir")
        .arg(&target_dir)
        .args(extra_args)
        .stdin(File::open(FIXTURE).unwrap())
        .env("SOURCE_DATE_EPOCH", "0")
        .env("RUST_LOG", "warn")
        .status()
        .unwrap();
    assert!(status.success());
    target_dir
}

/// Relative paths of all the files under `dir`, sorted
fn files(dir: &Path) -> Vec<PathBuf> {
    fn visit(base: &Path, dir: &Path, acc: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(base, &path, acc);
            } else {
                acc.push(path.strip_prefix(base).unwrap().to_path_buf());
            }
        }
    }
    let mut acc = vec![];
    if dir.exists() {
        visit(dir, dir, &mut acc);
    }
    acc.sort();
    acc
}

#[test]
fn site_snapshot() {
    let site = run("site_snapshot", &[]).join("site");
    let snapshot = Path::new(SNAPSHOT_DIR);
    let generated: Vec<_> = files(&site)
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "html" || e == "csv"))
        .collect();
    assert!(generated.len() >= 16, "too few pages: {:?}", generated);

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        if snapshot.exists() {
            fs::remove_dir_all(snapshot).unwrap();
        }
        for rel in generated.iter() {
            let dest = snapshot.join(rel);
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::copy(site.join(rel), dest).unwrap();
        }
        return;
    }

    assert_eq!(
        generated,
        files(snapshot),
        "generated pages differ from snapshot pages, run with UPDATE_SNAPSHOTS=1 to update"
    );
    for rel in generated.iter() {
        let got = fs::read_to_string(site.join(rel)).unwrap();
        let expected = fs::read_to_string(snapshot.join(rel)).unwrap();
        if got != expected {
            let pos = got
                .bytes()
                .zip(expected.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(got.len().min(expected.len()));
            let context = |s: &str| {
                s.get(pos.saturating_sub(80)..(pos + 80).min(s.len()))
                    .unwrap_or("")
                    .to_string()
            };
            panic!(
                "{} differs from snapshot at byte {}, run with UPDATE_SNAPSHOTS=1 to update\n got: {}\nexpected: {}",
                rel.display(),
                pos,
                context(&got),
                context(&expected),
            );
        }
    }
}

#[test]
fn sample_out() {
    let sample = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sample.bin");
    run(
        "sample_out",
        &[
            "--sample-every",
            "100",
            "--sample-out",
            sample.to_str().unwrap(),
        ],
    );
    let mut reader = &fs::read(&sample).unwrap()[..];
    let mut heights = vec![];
    while !reader.is_empty() {
        heights.push(BlockExtra::consensus_decode(&mut reader).unwrap().height());
    }
    assert_eq!(heights, vec![0, 100, 200, 300]);
}
