Build a portable Linux amd64 musl binary and package it as a .tar.gz archive.

Steps to execute using the Bash tool:

1. **Install musl development dependencies** if not already present:
```bash
dpkg -l musl-tools musl-dev &>/dev/null || sudo apt-get install -y musl-tools musl-dev
```

2. **Check and install the musl target** if not already present:
```bash
rustup target list --installed | grep -q x86_64-unknown-linux-musl || rustup target add x86_64-unknown-linux-musl
```

3. **Compile the release binary** for the musl target:
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

4. **Read the version** from Cargo.toml:
```bash
grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
```

5. **Create the output directory** if it does not exist:
```bash
mkdir -p .build
```

6. **Package the binary** as `ellie-<version>-linux-amd64.tar.gz` inside the `.build/` directory at the project root:
```bash
tar -czf .build/ellie-<version>-linux-amd64.tar.gz -C target/x86_64-unknown-linux-musl/release ellie
```

Run all steps sequentially. Report the final archive path and its size when done.
