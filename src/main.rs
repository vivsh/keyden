use argh::FromArgs;
use keyden::{KeyStoreError, KeyManager, FileKeyStore};
use std::env;

#[derive(FromArgs)]
/// Keyden: a lightweight CLI tool to manage, rotate, and inspect secret keys safely
pub struct KeydenArgs {
    #[argh(subcommand)]
    pub command: KeydenCommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum KeydenCommand {
    Rotate(RotateArgs),
    Current(CurrentArgs),
    List(ListArgs),
    Generate(GenerateArgs),
}

#[derive(FromArgs)]
/// Rotate keys: generate new ones if needed based on key count and TTL.
#[argh(subcommand, name = "rotate")]
pub struct RotateArgs {
    /// path to key file
    #[argh(positional)]
    pub file: Option<String>,

    /// size of each secret key
    #[argh(option, default = "128")]
    pub size: usize,
}

#[derive(FromArgs)]
/// Print the currently active secret key.
#[argh(subcommand, name = "current")]
pub struct CurrentArgs {
    /// path to key file
    #[argh(positional)]
    pub file: Option<String>,
}

#[derive(FromArgs)]
/// List all stored secret keys.
#[argh(subcommand, name = "list")]
pub struct ListArgs {
    /// path to key file
    #[argh(positional)]
    pub file: Option<String>,
}

#[derive(FromArgs)]
/// Generate a one-time secret key without storing it.
#[argh(subcommand, name = "generate")]
pub struct GenerateArgs {
    /// size of the secret key
    #[argh(option, default = "128")]
    pub size: usize,
}

/// Try to resolve file: positional argument > env KEYDEN_FILE > error
fn resolve_file(file_arg: &Option<String>) -> Result<String, KeyStoreError> {
    if let Some(path) = file_arg {
        return Ok(path.clone());
    }
    if let Ok(env_path) = env::var("KEYDEN_FILE") {
        return Ok(env_path);
    }
    Err(KeyStoreError::Other("Missing key file argument and $KEYDEN_FILE not set".into()))
}

fn main() -> Result<(), KeyStoreError> {
    let args: KeydenArgs = argh::from_env();

    match args.command {
        KeydenCommand::Rotate(args) => {
            let file = resolve_file(&args.file)?;
            let store = FileKeyStore::new(&file)?;
            let manager = KeyManager::builder(store)
                .size(args.size)
                .build()?;

            match manager.rotate_keys()? {
                true => println!("New key rotated successfully."),
                false => println!("No key rotation needed."),
            }
        }
        KeydenCommand::Current(args) => {
            let file = resolve_file(&args.file)?;
            let store = FileKeyStore::new(&file)?;
            let manager = KeyManager::builder(store).build()?;

            if let Some(key) = manager.current_key() {
                println!("{}", key.secret);
            } else {
                eprintln!("No active key found.");
            }
        }
        KeydenCommand::List(args) => {
            let file = resolve_file(&args.file)?;
            let store = FileKeyStore::new(&file)?;
            let manager = KeyManager::builder(store).build()?;

            for key in manager.list_keys() {
                println!("{} (created at {})", key.kid, key.created_at_unix);
            }
        }
        KeydenCommand::Generate(args) => {
            let key = KeyManager::generate_temp_key(args.size);
            println!("{}", key.secret);
        }
    }

    Ok(())
}