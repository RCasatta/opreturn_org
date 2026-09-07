use bitcoin::hashes::{sha256, Hash};
use bitcoin::BlockHash;
use std::convert::{TryFrom, TryInto};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

// Two adjacent cache lines keep block-load variance low without returning to fully random probes.
const BLOCK_BYTES: usize = 128;
const BLOCK_BITS: u64 = (BLOCK_BYTES * 8) as u64;
const BLOCKED_FILTER_OVERHEAD: f64 = 1.30;
const MAX_HASH_FUNCTIONS: u8 = 32;
const BASE_HEADER_LEN: usize = 128;
const DELTA_HEADER_LEN: usize = 160;
const BASE_MAGIC: &[u8; 8] = b"OSPKBF01";
const DELTA_MAGIC: &[u8; 8] = b"OSPKBD01";

#[derive(Debug, Clone)]
pub struct BloomConfig {
    pub directory: PathBuf,
    pub expected_items: u64,
    pub false_positive_rate: f64,
}

pub struct UsedScriptBloom {
    config: BloomConfig,
    bits: Vec<u8>,
    blocks: u64,
    hash_functions: u8,
    insertions: u64,
    set_bits: u64,
    tip: Option<(u32, BlockHash)>,
    building_base: bool,
    process_current_block: bool,
    delta_start: Option<(u32, BlockHash)>,
    new_positions: Vec<u64>,
}

impl UsedScriptBloom {
    pub fn new(config: BloomConfig) -> io::Result<Self> {
        validate_config(&config)?;
        fs::create_dir_all(&config.directory)?;

        let base_path = config.directory.join("base.bloom");
        if base_path.exists() {
            let mut bloom = Self::load_base(config, &base_path)?;
            bloom.load_deltas()?;
            let (height, hash) = bloom.tip.expect("a base Bloom filter always has a tip");
            log::info!(
                "loaded used scriptPubKeys Bloom filter: {} bytes, tip {height} {hash}",
                bloom.bits.len()
            );
            Ok(bloom)
        } else {
            if !delta_paths(&config.directory)?.is_empty() {
                return Err(invalid_data(
                    "Bloom delta files exist but base.bloom is missing; delete the state directory to rebuild",
                ));
            }

            let (bytes, blocks, hash_functions) = layout(&config)?;
            let bits = allocate_zeroed(bytes)?;
            log::info!(
                "initialized used scriptPubKeys Bloom base: {bytes} bytes, {hash_functions} hash functions"
            );
            Ok(Self {
                config,
                bits,
                blocks,
                hash_functions,
                insertions: 0,
                set_bits: 0,
                tip: None,
                building_base: true,
                process_current_block: false,
                delta_start: None,
                new_positions: Vec::new(),
            })
        }
    }

    fn load_base(config: BloomConfig, path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; BASE_HEADER_LEN];
        file.read_exact(&mut header)?;
        if &header[0..8] != BASE_MAGIC {
            return Err(invalid_data("invalid base.bloom magic or format version"));
        }

        let version = read_u16(&header, 8);
        let block_bits = read_u16(&header, 10);
        let hash_functions = header[12];
        let expected_items = read_u64(&header, 16);
        let false_positive_rate = f64::from_bits(read_u64(&header, 24));
        let blocks = read_u64(&header, 32);
        let insertions = read_u64(&header, 40);
        let set_bits = read_u64(&header, 48);
        let height = read_u32(&header, 56);
        let block_hash = BlockHash::from_byte_array(header[60..92].try_into().unwrap());
        let expected_bitmap_hash: [u8; 32] = header[92..124].try_into().unwrap();

        if version != 1 || u64::from(block_bits) != BLOCK_BITS {
            return Err(invalid_data("unsupported base.bloom layout"));
        }
        ensure_compatible(
            &config,
            expected_items,
            false_positive_rate,
            blocks,
            hash_functions,
        )?;
        if height == u32::MAX {
            return Err(invalid_data("base.bloom has no block tip"));
        }

        let bytes = blocks
            .checked_mul(BLOCK_BYTES as u64)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| invalid_data("base.bloom bitmap is too large"))?;
        let mut bits = allocate_zeroed(bytes)?;
        file.read_exact(&mut bits)?;
        let mut trailing = [0u8; 1];
        if file.read(&mut trailing)? != 0 {
            return Err(invalid_data("base.bloom contains trailing data"));
        }
        if sha256::Hash::hash(&bits).to_byte_array() != expected_bitmap_hash {
            return Err(invalid_data("base.bloom bitmap checksum mismatch"));
        }

        Ok(Self {
            config,
            bits,
            blocks,
            hash_functions,
            insertions,
            set_bits,
            tip: Some((height, block_hash)),
            building_base: false,
            process_current_block: false,
            delta_start: None,
            new_positions: Vec::new(),
        })
    }

    fn load_deltas(&mut self) -> io::Result<()> {
        for path in delta_paths(&self.config.directory)? {
            self.apply_delta(&path)?;
        }
        Ok(())
    }

    fn apply_delta(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        if bytes.len() < DELTA_HEADER_LEN || &bytes[0..8] != DELTA_MAGIC {
            return Err(invalid_data(format!(
                "invalid Bloom delta: {}",
                path.display()
            )));
        }
        if read_u16(&bytes, 8) != 1 || u64::from(read_u16(&bytes, 10)) != BLOCK_BITS {
            return Err(invalid_data(format!(
                "unsupported Bloom delta layout: {}",
                path.display()
            )));
        }
        let expected_items = read_u64(&bytes, 16);
        let false_positive_rate = f64::from_bits(read_u64(&bytes, 24));
        let blocks = read_u64(&bytes, 32);
        ensure_compatible(
            &self.config,
            expected_items,
            false_positive_rate,
            blocks,
            bytes[12],
        )?;

        let start_height = read_u32(&bytes, 40);
        let end_height = read_u32(&bytes, 44);
        let previous_hash = BlockHash::from_byte_array(bytes[48..80].try_into().unwrap());
        let end_hash = BlockHash::from_byte_array(bytes[80..112].try_into().unwrap());
        let position_count = read_u64(&bytes, 112);
        let payload_len = usize::try_from(read_u64(&bytes, 120))
            .map_err(|_| invalid_data("Bloom delta payload is too large"))?;
        let expected_payload_hash: [u8; 32] = bytes[128..160].try_into().unwrap();
        let payload_end = DELTA_HEADER_LEN
            .checked_add(payload_len)
            .ok_or_else(|| invalid_data("Bloom delta payload length overflow"))?;
        let payload = bytes
            .get(DELTA_HEADER_LEN..payload_end)
            .ok_or_else(|| invalid_data("truncated Bloom delta payload"))?;
        if bytes.len() != payload_end {
            return Err(invalid_data("Bloom delta contains trailing data"));
        }
        if sha256::Hash::hash(payload).to_byte_array() != expected_payload_hash {
            return Err(invalid_data("Bloom delta payload checksum mismatch"));
        }

        let (tip_height, tip_hash) = self.tip.expect("base was loaded before deltas");
        if start_height
            != tip_height
                .checked_add(1)
                .ok_or_else(|| invalid_data("height overflow"))?
            || previous_hash != tip_hash
            || end_height < start_height
        {
            return Err(invalid_data(format!(
                "Bloom delta does not follow the current tip: {}",
                path.display()
            )));
        }

        let positions = decode_positions(payload, position_count)?;
        for position in positions {
            self.set_position(position)?;
        }
        self.tip = Some((end_height, end_hash));
        Ok(())
    }

    pub fn begin_block(
        &mut self,
        height: u32,
        block_hash: BlockHash,
        previous_hash: BlockHash,
    ) -> io::Result<()> {
        self.process_current_block = false;
        match self.tip {
            None => {
                if height != 0 {
                    return Err(invalid_data(
                        "cannot create base.bloom because the input stream does not start at genesis",
                    ));
                }
                self.process_current_block = true;
                self.tip = Some((height, block_hash));
            }
            Some((tip_height, _)) if height < tip_height => {}
            Some((tip_height, tip_hash)) if height == tip_height => {
                if block_hash != tip_hash {
                    return Err(invalid_data(format!(
                        "stored Bloom tip hash differs from the input at height {height}; delete the state directory to rebuild"
                    )));
                }
            }
            Some((tip_height, tip_hash)) => {
                if height
                    != tip_height
                        .checked_add(1)
                        .ok_or_else(|| invalid_data("height overflow"))?
                    || previous_hash != tip_hash
                {
                    return Err(invalid_data(format!(
                        "block {height} does not follow stored Bloom tip {tip_height}; delete the state directory to rebuild"
                    )));
                }
                if !self.building_base && self.delta_start.is_none() {
                    self.delta_start = Some((height, tip_hash));
                }
                self.process_current_block = true;
                self.tip = Some((height, block_hash));
            }
        }
        Ok(())
    }

    pub fn insert(&mut self, script_pubkey: &[u8]) {
        if !self.process_current_block {
            return;
        }
        let digest = sha256::Hash::hash(script_pubkey).to_byte_array();
        let block_hash = u64::from_le_bytes(digest[0..8].try_into().unwrap());
        let first_bit = u64::from_le_bytes(digest[8..16].try_into().unwrap());
        let bit_step = u64::from_le_bytes(digest[16..24].try_into().unwrap()) | 1;
        let block_offset = (block_hash % self.blocks) * BLOCK_BITS;

        for index in 0..u64::from(self.hash_functions) {
            // The odd step visits every bit in this power-of-two-sized block before repeating.
            let bit = first_bit.wrapping_add(index.wrapping_mul(bit_step)) & (BLOCK_BITS - 1);
            let position = block_offset + bit;
            let byte_index = (position / 8) as usize;
            let mask = 1u8 << (position % 8);
            if self.bits[byte_index] & mask == 0 {
                self.bits[byte_index] |= mask;
                self.set_bits += 1;
                if !self.building_base {
                    self.new_positions.push(position);
                }
            }
        }
        self.insertions += 1;
    }

    #[cfg(test)]
    fn contains(&self, script_pubkey: &[u8]) -> bool {
        let digest = sha256::Hash::hash(script_pubkey).to_byte_array();
        let block_hash = u64::from_le_bytes(digest[0..8].try_into().unwrap());
        let first_bit = u64::from_le_bytes(digest[8..16].try_into().unwrap());
        let bit_step = u64::from_le_bytes(digest[16..24].try_into().unwrap()) | 1;
        let block_offset = (block_hash % self.blocks) * BLOCK_BITS;

        for index in 0..u64::from(self.hash_functions) {
            let bit = first_bit.wrapping_add(index.wrapping_mul(bit_step)) & (BLOCK_BITS - 1);
            let position = block_offset + bit;
            if self.bits[(position / 8) as usize] & (1u8 << (position % 8)) == 0 {
                return false;
            }
        }
        true
    }

    pub fn publish(&mut self) -> io::Result<()> {
        if self.building_base {
            self.publish_base()?;
            self.building_base = false;
        } else if self.delta_start.is_some() {
            self.publish_delta()?;
        }
        Ok(())
    }

    fn publish_base(&self) -> io::Result<()> {
        let (height, block_hash) = self
            .tip
            .ok_or_else(|| invalid_data("cannot publish base.bloom without any blocks"))?;
        let final_path = self.config.directory.join("base.bloom");
        let temporary = self.config.directory.join(".base.bloom.tmp");
        let bitmap_hash = sha256::Hash::hash(&self.bits).to_byte_array();
        let mut file = create_temporary(&temporary)?;
        file.write_all(&self.base_header(height, block_hash, bitmap_hash))?;
        file.write_all(&self.bits)?;
        publish_file(file, &temporary, &final_path, &self.config.directory)
    }

    fn publish_delta(&mut self) -> io::Result<()> {
        let (start_height, previous_hash) = self.delta_start.expect("checked by caller");
        let (end_height, end_hash) = self.tip.expect("new blocks update the tip");
        self.new_positions.sort_unstable();
        self.new_positions.dedup();
        let payload = encode_positions(&self.new_positions);
        let payload_hash = sha256::Hash::hash(&payload).to_byte_array();
        let file_name = format!("{start_height:010}-{end_height:010}-{end_hash}.delta");
        let final_path = self.config.directory.join(&file_name);
        let temporary = self.config.directory.join(format!(".{file_name}.tmp"));
        let mut file = create_temporary(&temporary)?;
        file.write_all(&self.delta_header(
            start_height,
            end_height,
            previous_hash,
            end_hash,
            self.new_positions.len() as u64,
            payload.len() as u64,
            payload_hash,
        ))?;
        file.write_all(&payload)?;
        publish_file(file, &temporary, &final_path, &self.config.directory)?;
        log::info!(
            "published Bloom delta {start_height}-{end_height}: {} positions, {} bytes",
            self.new_positions.len(),
            payload.len()
        );
        self.delta_start = None;
        self.new_positions.clear();
        Ok(())
    }

    fn base_header(&self, height: u32, block_hash: BlockHash, bitmap_hash: [u8; 32]) -> Vec<u8> {
        let mut header = common_header(BASE_MAGIC, self);
        header.extend_from_slice(&self.insertions.to_le_bytes());
        header.extend_from_slice(&self.set_bits.to_le_bytes());
        header.extend_from_slice(&height.to_le_bytes());
        header.extend_from_slice(&block_hash.to_byte_array());
        header.extend_from_slice(&bitmap_hash);
        header.resize(BASE_HEADER_LEN, 0);
        header
    }

    #[allow(clippy::too_many_arguments)]
    fn delta_header(
        &self,
        start_height: u32,
        end_height: u32,
        previous_hash: BlockHash,
        end_hash: BlockHash,
        position_count: u64,
        payload_len: u64,
        payload_hash: [u8; 32],
    ) -> Vec<u8> {
        let mut header = common_header(DELTA_MAGIC, self);
        header.extend_from_slice(&start_height.to_le_bytes());
        header.extend_from_slice(&end_height.to_le_bytes());
        header.extend_from_slice(&previous_hash.to_byte_array());
        header.extend_from_slice(&end_hash.to_byte_array());
        header.extend_from_slice(&position_count.to_le_bytes());
        header.extend_from_slice(&payload_len.to_le_bytes());
        header.extend_from_slice(&payload_hash);
        debug_assert_eq!(header.len(), DELTA_HEADER_LEN);
        header
    }

    fn set_position(&mut self, position: u64) -> io::Result<()> {
        if position >= self.bits.len() as u64 * 8 {
            return Err(invalid_data("Bloom delta bit position is out of range"));
        }
        let byte_index = (position / 8) as usize;
        let mask = 1u8 << (position % 8);
        if self.bits[byte_index] & mask == 0 {
            self.bits[byte_index] |= mask;
            self.set_bits += 1;
        }
        Ok(())
    }

    pub fn byte_len(&self) -> usize {
        self.bits.len()
    }

    pub fn hash_functions(&self) -> u8 {
        self.hash_functions
    }
}

fn validate_config(config: &BloomConfig) -> io::Result<()> {
    if config.expected_items == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Bloom expected items must be greater than zero",
        ));
    }
    if !(config.false_positive_rate > 0.0 && config.false_positive_rate < 1.0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Bloom false-positive rate must be greater than zero and less than one",
        ));
    }
    Ok(())
}

fn delta_paths(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path
            .extension()
            .map(|value| value == "delta")
            .unwrap_or(false)
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn layout(config: &BloomConfig) -> io::Result<(usize, u64, u8)> {
    let standard_bits = -(config.expected_items as f64 * config.false_positive_rate.ln())
        / std::f64::consts::LN_2.powi(2);
    let requested_bits = standard_bits * BLOCKED_FILTER_OVERHEAD;
    if !requested_bits.is_finite() || requested_bits > usize::MAX as f64 * 8.0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "requested Bloom filter is too large for this platform",
        ));
    }
    let blocks = (requested_bits / BLOCK_BITS as f64).ceil() as u64;
    let bytes = blocks
        .checked_mul(BLOCK_BYTES as u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "requested Bloom filter is too large for this platform",
            )
        })?;
    let standard_bits_per_item = standard_bits / config.expected_items as f64;
    let hash_functions = (standard_bits_per_item * std::f64::consts::LN_2)
        .round()
        .clamp(1.0, f64::from(MAX_HASH_FUNCTIONS)) as u8;
    Ok((bytes, blocks, hash_functions))
}

fn ensure_compatible(
    config: &BloomConfig,
    expected_items: u64,
    false_positive_rate: f64,
    blocks: u64,
    hash_functions: u8,
) -> io::Result<()> {
    let (_, configured_blocks, configured_hash_functions) = layout(config)?;
    if expected_items != config.expected_items
        || false_positive_rate.to_bits() != config.false_positive_rate.to_bits()
        || blocks != configured_blocks
        || hash_functions != configured_hash_functions
    {
        return Err(invalid_data(
            "Bloom parameters differ from the existing state; delete the state directory to rebuild",
        ));
    }
    Ok(())
}

fn allocate_zeroed(bytes: usize) -> io::Result<Vec<u8>> {
    let mut bits = Vec::new();
    bits.try_reserve_exact(bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::OutOfMemory,
            format!("cannot allocate {bytes} bytes for Bloom filter: {error}"),
        )
    })?;
    bits.resize(bytes, 0);
    Ok(bits)
}

fn common_header(magic: &[u8; 8], bloom: &UsedScriptBloom) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(magic);
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&(BLOCK_BITS as u16).to_le_bytes());
    header.push(bloom.hash_functions);
    header.extend_from_slice(&[0u8; 3]);
    header.extend_from_slice(&bloom.config.expected_items.to_le_bytes());
    header.extend_from_slice(&bloom.config.false_positive_rate.to_bits().to_le_bytes());
    header.extend_from_slice(&bloom.blocks.to_le_bytes());
    header
}

fn create_temporary(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
}

fn publish_file(
    mut file: File,
    temporary: &Path,
    final_path: &Path,
    directory: &Path,
) -> io::Result<()> {
    file.flush()?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, final_path)?;
    File::open(directory)?.sync_all()
}

fn encode_positions(positions: &[u64]) -> Vec<u8> {
    let mut payload = Vec::new();
    let mut previous = 0u64;
    for (index, position) in positions.iter().copied().enumerate() {
        let delta = if index == 0 {
            position
        } else {
            position - previous
        };
        write_varint(delta, &mut payload);
        previous = position;
    }
    payload
}

fn decode_positions(payload: &[u8], count: u64) -> io::Result<Vec<u64>> {
    if count > payload.len() as u64 {
        return Err(invalid_data(
            "Bloom delta position count exceeds its payload length",
        ));
    }
    let capacity = usize::try_from(count).map_err(|_| invalid_data("too many Bloom positions"))?;
    let mut positions = Vec::with_capacity(capacity);
    let mut offset = 0usize;
    let mut previous = 0u64;
    for index in 0..count {
        let delta = read_varint(payload, &mut offset)?;
        if index > 0 && delta == 0 {
            return Err(invalid_data(
                "Bloom delta positions are not strictly increasing",
            ));
        }
        let position = if index == 0 {
            delta
        } else {
            previous
                .checked_add(delta)
                .ok_or_else(|| invalid_data("Bloom delta position overflow"))?
        };
        positions.push(position);
        previous = position;
    }
    if offset != payload.len() {
        return Err(invalid_data(
            "Bloom delta position count does not match payload",
        ));
    }
    Ok(positions)
}

fn write_varint(mut value: u64, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn read_varint(input: &[u8], offset: &mut usize) -> io::Result<u64> {
    let mut value = 0u64;
    for shift in (0..=63).step_by(7) {
        let byte = *input
            .get(*offset)
            .ok_or_else(|| invalid_data("truncated Bloom delta varint"))?;
        *offset += 1;
        if shift == 63 && byte > 1 {
            return Err(invalid_data("Bloom delta varint overflow"));
        }
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(invalid_data("Bloom delta varint overflow"))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub fn parse_false_positive_rate(value: &str) -> Result<f64, String> {
    let value = value
        .parse::<f64>()
        .map_err(|error| format!("invalid false-positive rate: {error}"))?;
    if value > 0.0 && value < 1.0 {
        Ok(value)
    } else {
        Err("false-positive rate must be greater than zero and less than one".to_string())
    }
}

pub fn parse_positive_u64(value: &str) -> Result<u64, String> {
    let value = value
        .parse::<u64>()
        .map_err(|error| format!("invalid positive integer: {error}"))?;
    if value > 0 {
        Ok(value)
    } else {
        Err("value must be greater than zero".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("opreturn-org-bloom-{}-{name}", std::process::id()))
    }

    fn new_filter(name: &str, expected_items: u64, false_positive_rate: f64) -> UsedScriptBloom {
        let directory = test_directory(name);
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        UsedScriptBloom::new(BloomConfig {
            directory,
            expected_items,
            false_positive_rate,
        })
        .unwrap()
    }

    fn begin_genesis(bloom: &mut UsedScriptBloom) {
        bloom
            .begin_block(0, BlockHash::all_zeros(), BlockHash::all_zeros())
            .unwrap();
    }

    #[test]
    fn rejects_invalid_configuration() {
        assert!(parse_false_positive_rate("0").is_err());
        assert!(parse_false_positive_rate("1").is_err());
        assert!(parse_false_positive_rate("NaN").is_err());
        assert!(parse_false_positive_rate("0.001").is_ok());
        assert!(parse_positive_u64("0").is_err());
        assert!(parse_positive_u64("5000000000").is_ok());
    }

    #[test]
    fn inserted_items_are_present() {
        let mut bloom = new_filter("inserted", 1_000, 0.001);
        begin_genesis(&mut bloom);
        for value in 0..1_000u64 {
            bloom.insert(&value.to_le_bytes());
        }
        for value in 0..1_000u64 {
            assert!(bloom.contains(&value.to_le_bytes()));
        }
        fs::remove_dir_all(&bloom.config.directory).unwrap();
    }

    #[test]
    fn observed_false_positive_rate_is_below_target() {
        let mut bloom = new_filter("false-positive", 10_000, 0.001);
        begin_genesis(&mut bloom);
        for value in 0..10_000u64 {
            bloom.insert(&value.to_le_bytes());
        }

        let false_positives = (10_000..110_000u64)
            .filter(|value| bloom.contains(&value.to_le_bytes()))
            .count();
        assert!(
            false_positives <= 100,
            "false positives: {}",
            false_positives
        );
        fs::remove_dir_all(&bloom.config.directory).unwrap();
    }

    #[test]
    fn sizing_includes_blocked_filter_headroom() {
        let bloom = new_filter("sizing", 5_000, 0.001);
        assert!(bloom.byte_len() >= 11_500);
        assert!(bloom.byte_len() <= 12_000);
        assert_eq!(bloom.hash_functions(), 10);
        fs::remove_dir_all(&bloom.config.directory).unwrap();
    }

    #[test]
    fn base_and_delta_round_trip() {
        let directory = test_directory("round-trip");
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        let config = BloomConfig {
            directory: directory.clone(),
            expected_items: 100,
            false_positive_rate: 0.001,
        };

        let mut initial = UsedScriptBloom::new(config.clone()).unwrap();
        begin_genesis(&mut initial);
        initial.insert(b"first script");
        initial.publish().unwrap();
        assert!(directory.join("base.bloom").is_file());

        let mut updated = UsedScriptBloom::new(config.clone()).unwrap();
        updated
            .begin_block(0, BlockHash::all_zeros(), BlockHash::all_zeros())
            .unwrap();
        let next_hash = BlockHash::from_byte_array([1u8; 32]);
        updated
            .begin_block(1, next_hash, BlockHash::all_zeros())
            .unwrap();
        updated.insert(b"second script");
        updated.publish().unwrap();
        assert_eq!(
            fs::read_dir(&directory)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .map(|value| value == "delta")
                        .unwrap_or(false)
                })
                .count(),
            1
        );

        let loaded = UsedScriptBloom::new(config).unwrap();
        assert!(loaded.contains(b"first script"));
        assert!(loaded.contains(b"second script"));
        assert_eq!(loaded.tip, Some((1, next_hash)));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn position_delta_encoding_round_trips() {
        let positions = vec![0, 1, 127, 128, 10_000, u32::MAX as u64];
        let payload = encode_positions(&positions);
        assert_eq!(
            decode_positions(&payload, positions.len() as u64).unwrap(),
            positions
        );
    }
}
