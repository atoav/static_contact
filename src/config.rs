//! The [Config] struct is the deserialization of the `config.toml` file.
//! 
//! Read the config from it's default path by running:
//! ```
//! let config = Config::new();
//! ```
//! 

use std::path::PathBuf;
use serde::Deserialize;
use std::fs;
use lazy_static::lazy_static;


lazy_static! {
    static ref CONFIG_PATH: PathBuf = PathBuf::from("/etc/static_contact/config.toml");
}

/// Stores server and endpoints configuration, read from `config.toml`
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub server: ServerConfig,
    pub endpoints: Vec<EndpointConfig>
}

/// Server and SMTP configuration
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ServerConfig {
    pub ip: String,
    pub port: usize,
    pub max_payload: usize,
    pub smtp_server: String,
    pub smtp_user: String,
    pub smtp_password: String
}

/// Endpoint configuration (multiple endpoints can exist)
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct EndpointConfig {
    pub identifier: String,
    pub name: String,
    pub domain: String,
    pub from_email: String,
    pub max_message_length: usize,
    pub max_name_length: usize,
    pub target: TargetConfig
}

/// Target Email configuration (every endpoint can have a target)
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TargetConfig {
    pub email: String,
    pub email_name: String
}


impl Config{
    /// Create a new `Config` by reading and deserializing from the path provided by the [get_app_root] function. This panics if no config path could be aquired or if the config at the given path cannot be read.
    /// ```
    /// let config = Config::new();
    /// ```
    pub fn new() -> Self {
        let contents = match fs::read_to_string(CONFIG_PATH.to_path_buf()){
            Ok(c) => c,
            Err(e) => {
                match e.kind() {
                   std::io::ErrorKind::PermissionDenied => {
                        match fs::metadata(&CONFIG_PATH.to_path_buf()) {
                            Ok(meta) => {
                                if meta.is_dir() {
                                    eprintln!("Error: cannot read the config at {:?} it seems to be a dir not a file...", CONFIG_PATH.to_path_buf());
                                }else{
                                    let permissions = meta.permissions();
                                    if cfg!(unix) {
                                        use std::os::unix::fs::PermissionsExt;
                                        eprintln!("Error: insufficient permissions to read the config at {:?} ({:o})", CONFIG_PATH.to_path_buf(), permissions.mode());
                                    }else{
                                        eprintln!("Error: insufficient permissions to read the config at {:?} (Readonly: {})", CONFIG_PATH.to_path_buf(), permissions.readonly());
                                    }
                                }
                                
                            },
                            Err(_e) => {
                                eprintln!("Error: insufficient permissions to read the config at {:?}", CONFIG_PATH.to_path_buf());
                            }
                        }
                    },
                    std::io::ErrorKind::NotFound => {
                        eprintln!("Error: No config.toml found at {:?}", CONFIG_PATH.to_path_buf());
                    }
                    _ => {
                        eprintln!("Error while attempting to read config from path {:?}: {:?}", CONFIG_PATH.to_path_buf(), e);
                    }
                }
                std::process::exit(0);
            }
        };

        toml::from_str(&contents).unwrap()
    }

    /// Return the path where a configuration is stored
    /// ```
    /// let config = Config::new();
    /// let path = config.path();
    /// println!("{:?}", path);
    /// ```
    pub fn path(&self) -> PathBuf {
        CONFIG_PATH.to_path_buf()
    }
}