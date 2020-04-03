//! The [Config] struct is the deserialization of the `config.toml` file. The path to the config is automatically chosen by the `get_app_root()` function, based on the OS and the XDG environment variables (if you use them). 
//! 
//! Read the config from it's default path by running:
//! ```
//! let config = Config::new();
//! ```
//! 

use app_dirs::*;
use std::path::PathBuf;
use serde::Deserialize;
use std::fs;

/// This reads the Appinfo from the `Cargo.toml` and passes ot to the `get_app_root()` command, so the propper config path can be generated.
const APP_INFO: AppInfo = AppInfo{name: env!("CARGO_PKG_NAME"), author: env!("CARGO_PKG_AUTHORS")};

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
        let path = get_app_root(AppDataType::UserConfig, &APP_INFO).expect("Error: Couldn't get a config path");
        let mut path: PathBuf = path.into();
        path.push("config.toml");

        let contents = match fs::read_to_string(&path){
            Ok(c) => c,
            Err(e) => {
                match e.kind() {
                   std::io::ErrorKind::PermissionDenied => {
                        match fs::metadata(&path) {
                            Ok(meta) => {
                                if meta.is_dir() {
                                    eprintln!("Error: cannot read the config at {:?} it seems to be a dir not a file...", path);
                                }else{
                                    let permissions = meta.permissions();
                                    if cfg!(unix) {
                                        use std::os::unix::fs::PermissionsExt;
                                        eprintln!("Error: insufficient permissions to read the config at {:?} ({:o})", path, permissions.mode());
                                    }else{
                                        eprintln!("Error: insufficient permissions to read the config at {:?} (Readonly: {})", path, permissions.readonly());
                                    }
                                }
                                
                            },
                            Err(_e) => {
                                eprintln!("Error: insufficient permissions to read the config at {:?}", path);
                            }
                        }
                    },
                    std::io::ErrorKind::NotFound => {
                        eprintln!("Error: No config.toml found at {:?}", path);
                    }
                    _ => {
                        eprintln!("Error while attempting to read config from path {:?}: {:?}", path, e);
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
        let path = get_app_root(AppDataType::UserConfig, &APP_INFO).expect("Error: Couldn't get a config path");
        let mut path: PathBuf = path.into();
        path.push("config.toml");

        path
    }
}