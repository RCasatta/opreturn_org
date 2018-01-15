# Rustat

Generate statistic charts about the Bitcoin blockchain.

# Dependency

You need [Rust](https://www.rust-lang.org/)
It is based on the ouput of the [bitcoin-iterate](https://github.com/rustyrussell/bitcoin-iterate) tool.

# Installation

```
git clone https://github.com/RCasatta/rustat
cd rustat
cargo build --release
```

# Usage

```
cd rustat
$PATH-TO-BITCOIN-ITERATE/bitcoin-iterate -q --output '%bs %os' | ./target/release/rustat
```

It takes about an hour to scan all the bitcoin blockchain on a quite shitty machine as of the 15 January


