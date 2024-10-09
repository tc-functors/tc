# Installation

Download the executable for your OS

| GNU/Linux x86                                                                   | MacOSX M1/M2                                                       | MacOSX x86                                                                      |
|---------------------------------------------------------------------------------|--------------------------------------------------------------------|---------------------------------------------------------------------------------|
| [0.7.0](https://github.com/informed-labs/tc/releases/download/0.7.0/tc-x86_64-linux) | [0.7.0](https://github.com/informed-labs/tc/releases/download/0.7.0/tc) | [0.7.0](https://github.com/informed-labs/tc/releases/download/0.7.0/tc-x86_64-apple) |


```admonish warning title="For Mac users"
Allow tc in Privacy & Security

The first time you run the downloaded executable you will get a popup that says it may be "malicious software"

Do the following:
* Go to `Privacy & Security` panel to the `Security/Settings` section
* Should have `App Store and identified developers` selected
* Where it says `tc was blocked from use becasue it is not from an identified developer`
    * Click on `Allow Anyway`

mv ~/Downloads/tc /usr/local/bin/tc

chmod +x /usr/local/bin/tc

```

### Building your own

`tc` is written in [Rust](https://www.youtube.com/watch?v=ul9vyWuT8SU).

If you prefer to build `tc` yourself, install rustc/cargo.

Install Cargo/Rust https://www.rust-lang.org/tools/install

```sh
cd tc
cargo build --release
sudo mv target/release/tc /usr/local/bin/tc
```
