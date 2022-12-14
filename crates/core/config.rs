use crate::StorageType;

impl Default for StorageOptions {
    fn default() -> Self {
        StorageOptions::Redis(RedisStorageOptions::default())
    }
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct IdentityConfig {
    pub signup_with_email: bool,
    pub signup_with_username: bool,
    pub signup_require_username: bool,
    pub signup_require_email: bool,
    pub signup_require_email_verification: bool,
    pub signup_process_lifetime: i64,

    pub login_with_username: bool,
    pub login_with_email: bool,
    pub login_process_lifetime: i64,

    pub allow_multiple_emails: bool,

    pub password_min_length: usize,
    pub check_leaked_passwords: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum StorageOptions {
    SQL(SQLStorageOptions),
    Redis(RedisStorageOptions),
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct SQLStorageOptions {
    pub storage_path: String,
    pub sql_url: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default)]
pub struct RedisStorageOptions {
    pub storage_path: String,
    pub redis_url: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default)]
pub struct TokenConfig {
    /// Sign JWT tokens
    pub sign_jwt: bool,

    /// access token lifetime in seconds
    pub access_token_lifetime: i64,

    /// refresh token lifetime in seconds
    pub refresh_token_lifetime: i64,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// the domain keygate is running on, e.g `accounts.example.com`
    /// refresh tokens are only valid for this domain
    pub keygate_domain: String,

    /// admin api port
    /// if set to 0, the admin api will not be available
    pub admin_port: u16,

    /// admin api interface
    pub admin_interface: String,

    /// admin api prefix
    pub admin_prefix: Option<String>,

    /// public api port
    /// if set to 0, the api will not be available
    pub public_port: u16,

    /// public api interface
    pub public_interface: String,

    /// public api prefix
    pub public_prefix: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub environment: Environment,

    /// What storage backend to use
    pub storage_type: StorageType,

    /// Options for the storage backend
    pub storage_options: StorageOptions,

    /// server configuration
    pub server: ServerConfig,

    /// token configuration
    pub token: TokenConfig,

    /// identity configuration
    pub identity: IdentityConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            admin_port: default_admin_port(),
            admin_interface: default_admin_interface(),
            public_port: default_public_port(),
            public_interface: default_public_interface(),
            keygate_domain: "auth.localhost".to_string(),
            admin_prefix: None,
            public_prefix: None,
        }
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            allow_multiple_emails: false,
            signup_with_email: false,
            signup_with_username: true,
            signup_require_username: true,
            signup_require_email: false,
            signup_require_email_verification: false,
            signup_process_lifetime: 60 * 60,
            login_with_email: true,
            login_with_username: true,
            login_process_lifetime: 60 * 60,
            check_leaked_passwords: true,
            password_min_length: 8,
        }
    }
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            sign_jwt: false,
            access_token_lifetime: default_access_token_lifetime(),
            refresh_token_lifetime: default_refresh_token_lifetime(),
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            storage_type: default_storage_type(),
            storage_options: default_storage_options(),
            environment: Environment::Development,
            server: ServerConfig::default(),
            token: TokenConfig::default(),
            identity: IdentityConfig::default(),
        }
    }
}

impl Default for SQLStorageOptions {
    fn default() -> Self {
        SQLStorageOptions {
            storage_path: default_storage_path(),
            sql_url: "sqlite://data.db".to_string(),
        }
    }
}

impl Default for RedisStorageOptions {
    fn default() -> Self {
        RedisStorageOptions {
            storage_path: default_storage_path(),
            redis_url: "redis://localhost/".to_string(),
        }
    }
}

fn default_access_token_lifetime() -> i64 {
    30 * 60
}

fn default_refresh_token_lifetime() -> i64 {
    14 * 24 * 3600
}

fn default_admin_port() -> u16 {
    8081
}

fn default_public_port() -> u16 {
    8080
}

fn default_admin_interface() -> String {
    "127.0.0.1".to_string()
}

fn default_public_interface() -> String {
    "0.0.0.0".to_string()
}

fn default_storage_type() -> StorageType {
    StorageType::SQL
}

fn default_storage_path() -> String {
    "./data".to_string()
}

fn default_environment() -> Environment {
    Environment::Development
}

fn default_storage_options() -> StorageOptions {
    StorageOptions::default()
}
