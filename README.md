[![Crates.io](https://img.shields.io/crates/v/keyden.svg)](https://crates.io/crates/keyden)

# Keyden

**Keyden** is a lightweight tool for **managing**, **rotating**, and **retrieving** secret keys.  
It is designed to be both a **fast command-line utility** and a **pure Rust library** — making secure key management simple for applications and developers.

---

## ✨ Features

- 🔑 **Secure Secret Management**: Manage printable secret keys for sessions, tokens, password resets.
- 🔄 **Key Rotation**: Rotate keys based on age or minimum key count.
- ⚡ **Blazing Fast**: Pure `std`, no async runtime needed.
- 🧹 **Minimalistic CLI**: Clean commands for scripting and automation.
- 📚 **Library First**: Fully embeddable inside any Rust server, CLI or worker.
- 🛠️ **Transparent Format**: Human-readable key files, simple for backups.

---

## 🭹 Design Philosophy

Keyden is built to be:

- **Sync-first**: No async or futures overhead unless necessary.
- **Memory efficient**: Minimal allocations, small footprint.
- **Concurrency safe**: Internal fast `RwLock` with `parking_lot`.
- **Extensible**: Plug different backends easily (e.g., database, encrypted stores).
- **Transparent and Simple**: Files are plain, portable, and auditable.

---

## 📦 Installation

Install Keyden CLI:

```bash
cargo install keyden
```

Or include it in your project:

```toml
[dependencies]
keyden = "0.1"
```

---

## 🛠️ CLI Usage

Keyden CLI manages secret keys through four subcommands:

| Subcommand | Description |
|:-----------|:------------|
| `rotate [file]` | Rotate keys. Generates new ones if count is insufficient. |
| `current [file]` | Print the latest active secret key. |
| `list [file]` | List all keys and their creation timestamps. |
| `generate` | Generate a one-time temporary secret key (not stored). |

### 🔹 Examples

Rotate keys in a file:

```bash
keyden rotate ./keys.txt
```

Rotate with custom key size:

```bash
keyden rotate ./keys.txt --size 256
```

Retrieve the current active key:

```bash
keyden current ./keys.txt
```

List all keys:

```bash
keyden list ./keys.txt
```

Generate a temporary secret key without storing:

```bash
keyden generate --size 512
```

### 🔹 Environment variable: `KEYDEN_FILE`

Instead of providing `[file]` every time, you can set a default file path:

```bash
export KEYDEN_FILE=./keys.txt
keyden rotate
keyden list
keyden current
```

If the positional file argument is missing, Keyden automatically tries `$KEYDEN_FILE`.

---

## 📚 Using Keyden as a Library

### Add to your `Cargo.toml`:

```toml
[dependencies]
keyden = "0.1"
```

### Example Rust usage

```rust
use keyden::file_store::FileKeyStore;
use keyden::key_manager::KeyManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a file-backed key store
    let store = FileKeyStore::new("./keys.txt")?;

    // Build the manager
    let manager = KeyManager::builder(store)
        .size(128)
        .count(2)
        .ttl_secs(86400) // 1 day
        .build()?;

    // Rotate if needed
    manager.rotate_keys()?;

    // Get current key
    if let Some(current) = manager.current_key() {
        println!("Current key: {}", current.secret);
    }

    // Generate a temporary one-time key
    let temp = KeyManager::generate_temp_key(256);
    println!("Temporary key: {}", temp.secret);

    Ok(())
}
```

---

## 🔒 Important Security Notes

- **Keyden does not encrypt key files**. Set correct permissions (`chmod 600 keys.txt`).
- The file format is portable and easy to parse in any environment.
- Rotate keys regularly for high-security environments.

---

## 🛠️ Project Structure

```plaintext
keyden/
├── src/
│   ├── main.rs        # CLI entry point
│   ├── lib.rs         # Library exports
│   ├── commons.rs     # Common types: KeyStore, KeyMaterial
│   ├── file_store.rs  # File-based KeyStore backend
│   ├── key_manager.rs # KeyManager: rotation, reload, listing
│   ├── utils.rs       # Helpers (e.g., generate_secret)
├── Cargo.toml
├── README.md
├── .gitignore
├── LICENSE
```

---

## 📜 License

Licensed under either of:

- MIT License
- Apache 2.0 License

at your option.

---
