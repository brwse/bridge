use core::time::Duration;
use std::{fmt, str::FromStr};

use genawaiter::sync::Gen;
use rand::{rng, seq::SliceRandom as _};

/// SSL modes for PostgreSQL connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl FromStr for SslMode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disable" => Ok(SslMode::Disable),
            "allow" => Ok(SslMode::Allow),
            "prefer" => Ok(SslMode::Prefer),
            "require" => Ok(SslMode::Require),
            "verify-ca" => Ok(SslMode::VerifyCa),
            "verify-full" => Ok(SslMode::VerifyFull),
            _ => Err(ParseError::InvalidSslMode(s.to_string())),
        }
    }
}

impl fmt::Display for SslMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SslMode::Disable => write!(f, "disable"),
            SslMode::Allow => write!(f, "allow"),
            SslMode::Prefer => write!(f, "prefer"),
            SslMode::Require => write!(f, "require"),
            SslMode::VerifyCa => write!(f, "verify-ca"),
            SslMode::VerifyFull => write!(f, "verify-full"),
        }
    }
}

/// Target session attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSessionAttrs {
    Any,
    ReadWrite,
    ReadOnly,
    Primary,
    Standby,
    PreferStandby,
}

impl FromStr for TargetSessionAttrs {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "any" => Ok(TargetSessionAttrs::Any),
            "read-write" => Ok(TargetSessionAttrs::ReadWrite),
            "read-only" => Ok(TargetSessionAttrs::ReadOnly),
            "primary" => Ok(TargetSessionAttrs::Primary),
            "standby" => Ok(TargetSessionAttrs::Standby),
            "prefer-standby" => Ok(TargetSessionAttrs::PreferStandby),
            _ => Err(ParseError::InvalidTargetSessionAttrs(s.to_string())),
        }
    }
}

/// Channel binding modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelBinding {
    Disable,
    Prefer,
    Require,
}

impl FromStr for ChannelBinding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disable" => Ok(ChannelBinding::Disable),
            "prefer" => Ok(ChannelBinding::Prefer),
            "require" => Ok(ChannelBinding::Require),
            _ => Err(ParseError::InvalidChannelBinding(s.to_string())),
        }
    }
}

/// Load balancing modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalanceHosts {
    Disable,
    Random,
}

impl FromStr for LoadBalanceHosts {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disable" => Ok(LoadBalanceHosts::Disable),
            "random" => Ok(LoadBalanceHosts::Random),
            _ => Err(ParseError::InvalidLoadBalanceMode(s.to_string())),
        }
    }
}

/// SSL certificate modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslCertMode {
    Disable,
    Allow,
    Require,
}

impl FromStr for SslCertMode {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disable" => Ok(SslCertMode::Disable),
            "allow" => Ok(SslCertMode::Allow),
            "require" => Ok(SslCertMode::Require),
            _ => Err(ParseError::InvalidSslCertMode(s.to_string())),
        }
    }
}

/// Parse errors for connection strings
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidSslMode(String),
    InvalidTargetSessionAttrs(String),
    InvalidChannelBinding(String),
    InvalidLoadBalanceMode(String),
    InvalidSslCertMode(String),
    InvalidPort(String),
    InvalidTimeout(String),
    InvalidBoolean(String),
    InvalidInteger(String),
    MissingValue(String),
    InvalidUri(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat(s) => write!(f, "Invalid connection string format: {s}"),
            ParseError::InvalidSslMode(s) => write!(f, "Invalid SSL mode: {s}"),
            ParseError::InvalidTargetSessionAttrs(s) => {
                write!(f, "Invalid target session attributes: {s}")
            }
            ParseError::InvalidChannelBinding(s) => write!(f, "Invalid channel binding mode: {s}"),
            ParseError::InvalidLoadBalanceMode(s) => write!(f, "Invalid load balance mode: {s}"),
            ParseError::InvalidSslCertMode(s) => write!(f, "Invalid SSL certificate mode: {s}"),
            ParseError::InvalidPort(s) => write!(f, "Invalid port: {s}"),
            ParseError::InvalidTimeout(s) => write!(f, "Invalid timeout: {s}"),
            ParseError::InvalidBoolean(s) => write!(f, "Invalid boolean value: {s}"),
            ParseError::InvalidInteger(s) => write!(f, "Invalid integer value: {s}"),
            ParseError::MissingValue(s) => write!(f, "Missing value for parameter: {s}"),
            ParseError::InvalidUri(s) => write!(f, "Invalid URI format: {s}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// PostgreSQL connection configuration
#[derive(Debug, Clone, Default)]
pub struct Config {
    // Connection parameters
    /// Name of host to connect to. Supports comma-separated list for multiple
    /// hosts. Default: localhost
    pub host: Vec<String>,

    /// Port number to connect to at the server host.
    /// Default: 5432
    pub port: Vec<u16>,

    /// The database name to connect to.
    /// Default: Same as user name
    pub dbname: Option<String>,

    /// PostgreSQL user name to connect as.
    /// Default: postgres
    pub user: Option<String>,

    /// Password to be used if the server demands password authentication.
    /// Use with caution - prefer secure authentication methods when possible.
    pub password: Option<String>,

    /// Name of file used to store passwords.
    /// Default: ~/.pgpass
    pub passfile: Option<String>,

    /// Maximum time to wait while connecting, in seconds.
    /// Default: Infinite wait (0 means no timeout)
    pub connect_timeout: Option<u32>,

    /// Value for the application_name configuration parameter.
    /// Useful for monitoring and logging to identify connections.
    pub application_name: Option<String>,

    // SSL parameters
    /// Determines SSL connection priority and certificate verification level.
    /// Options: disable, allow, prefer (default), require, verify-ca,
    /// verify-full
    pub sslmode: Option<SslMode>,

    /// SSL certificate authority (CA) certificates file.
    /// Default: ~/.postgresql/root.crt
    /// Special value "system" uses SSL implementation's trusted roots.
    pub sslrootcert: Option<String>,

    /// Controls SSL negotiation.
    /// Options: postgres, direct
    /// Default: postgres
    pub sslnegotiation: Option<String>,

    // Authentication parameters
    /// Specifies required authentication method from server.
    /// Options: password, md5, scram-sha-256, none
    /// Can use comma-separated list or negation with !
    pub require_auth: Option<String>,

    /// Controls client's use of channel binding for authentication.
    /// Options: require, prefer (default), disable
    pub channel_binding: Option<ChannelBinding>,

    // Advanced parameters
    /// Determines acceptable session properties for connection.
    /// Options: any (default), read-write, read-only, primary, standby,
    /// prefer-standby
    pub target_session_attrs: Option<TargetSessionAttrs>,

    /// Controls connection order across multiple hosts.
    /// Options: disable (default), random
    /// Random mode helps distribute connections across PostgreSQL servers.
    pub load_balance_hosts: Option<LoadBalanceHosts>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse URI format connection string
    fn parse_uri(uri: &str) -> Result<Self, ParseError> {
        let mut config = Config::new();

        // Remove the scheme
        let uri = if let Some(stripped) = uri.strip_prefix("postgresql://") {
            stripped
        } else if let Some(stripped) = uri.strip_prefix("postgres://") {
            stripped
        } else {
            return Err(ParseError::InvalidUri(uri.to_string()));
        };

        // Split into authority and path+query
        let (authority, path_and_query) = if let Some(pos) = uri.find('/') {
            (&uri[..pos], &uri[pos..])
        } else if let Some(pos) = uri.find('?') {
            (&uri[..pos], &uri[pos..])
        } else {
            (uri, "")
        };

        // Parse authority (user:password@host:port)
        if !authority.is_empty() {
            let (userinfo, hostport) = if let Some(pos) = authority.rfind('@') {
                (Some(&authority[..pos]), &authority[pos + 1..])
            } else {
                (None, authority)
            };

            // Parse user info
            if let Some(userinfo) = userinfo {
                let (user, password) = if let Some(pos) = userinfo.find(':') {
                    (&userinfo[..pos], Some(&userinfo[pos + 1..]))
                } else {
                    (userinfo, None)
                };

                if !user.is_empty() {
                    config.user = Some(
                        urlencoding::decode(user)
                            .map_err(|_| ParseError::InvalidUri(uri.to_string()))?
                            .into_owned(),
                    );
                }
                if let Some(pass) = password {
                    config.password = Some(
                        urlencoding::decode(pass)
                            .map_err(|_| ParseError::InvalidUri(uri.to_string()))?
                            .into_owned(),
                    );
                }
            }

            // Parse host and port
            if !hostport.is_empty() {
                // Handle IPv6 addresses
                if hostport.starts_with('[') {
                    if let Some(end) = hostport.find(']') {
                        let host = &hostport[1..end];
                        config.host.push(host.to_string());

                        let remainder = &hostport[end + 1..];
                        if let Some(port_str) = remainder.strip_prefix(':') {
                            let port = port_str
                                .parse::<u16>()
                                .map_err(|_| ParseError::InvalidPort(port_str.to_string()))?;
                            config.port.push(port);
                        }
                    } else {
                        return Err(ParseError::InvalidUri(uri.to_string()));
                    }
                } else {
                    // Regular host:port or just host
                    let parts: Vec<&str> = hostport.split(':').collect();
                    match parts.len() {
                        1 => config.host.push(parts[0].to_string()),
                        2 => {
                            config.host.push(parts[0].to_string());
                            let port = parts[1]
                                .parse::<u16>()
                                .map_err(|_| ParseError::InvalidPort(parts[1].to_string()))?;
                            config.port.push(port);
                        }
                        _ => return Err(ParseError::InvalidUri(uri.to_string())),
                    }
                }
            }
        }

        // Parse path (database name)
        if let Some(pos) = path_and_query.find('?') {
            let path = &path_and_query[..pos];
            if path.len() > 1 {
                config.dbname = Some(
                    urlencoding::decode(&path[1..])
                        .map_err(|_| ParseError::InvalidUri(uri.to_string()))?
                        .into_owned(),
                );
            }

            // Parse query parameters
            let query = &path_and_query[pos + 1..];
            for param in query.split('&') {
                if let Some(eq_pos) = param.find('=') {
                    let key = &param[..eq_pos];
                    let value = urlencoding::decode(&param[eq_pos + 1..])
                        .map_err(|_| ParseError::InvalidUri(uri.to_string()))?;
                    config.set_param(key, &value)?;
                }
            }
        } else if path_and_query.len() > 1 {
            config.dbname = Some(
                urlencoding::decode(&path_and_query[1..])
                    .map_err(|_| ParseError::InvalidUri(uri.to_string()))?
                    .into_owned(),
            );
        }

        Ok(config)
    }

    /// Parse key-value format connection string
    fn parse_key_value(s: &str) -> Result<Self, ParseError> {
        let mut config = Config::new();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_value = false;
        let mut in_quotes = false;
        let mut escape_next = false;

        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if escape_next {
                if in_value {
                    current_value.push(ch);
                } else {
                    current_key.push(ch);
                }
                escape_next = false;
            } else if ch == '\\' && i + 1 < chars.len() {
                escape_next = true;
            } else if ch == '\'' && in_value {
                in_quotes = !in_quotes;
            } else if ch == '=' && !in_value && !in_quotes {
                in_value = true;
            } else if ch.is_whitespace() && !in_quotes {
                if in_value && !current_key.is_empty() {
                    config.set_param(&current_key, &current_value)?;
                    current_key.clear();
                    current_value.clear();
                    in_value = false;
                }
            } else if in_value {
                current_value.push(ch);
            } else {
                current_key.push(ch);
            }

            i += 1;
        }

        // Handle last parameter
        if !current_key.is_empty() {
            config.set_param(&current_key, &current_value)?;
        }

        Ok(config)
    }

    /// Set a parameter value
    fn set_param(&mut self, key: &str, value: &str) -> Result<(), ParseError> {
        match key {
            // Connection parameters
            "host" => {
                self.host = value.split(',').map(|s| s.to_string()).collect();
            }
            "port" => {
                self.port = value
                    .split(',')
                    .map(|s| s.parse::<u16>().map_err(|_| ParseError::InvalidPort(s.to_string())))
                    .collect::<Result<Vec<_>, _>>()?;
            }
            "dbname" => self.dbname = Some(value.to_string()),
            "user" => self.user = Some(value.to_string()),
            "password" => self.password = Some(value.to_string()),
            "passfile" => self.passfile = Some(value.to_string()),
            "connect_timeout" => {
                self.connect_timeout =
                    Some(value.parse().map_err(|_| ParseError::InvalidTimeout(value.to_string()))?);
            }
            "application_name" => self.application_name = Some(value.to_string()),

            // SSL parameters
            "sslmode" => self.sslmode = Some(value.parse()?),
            "sslrootcert" => self.sslrootcert = Some(value.to_string()),
            "sslnegotiation" => self.sslnegotiation = Some(value.to_string()),

            // Authentication parameters
            "require_auth" => self.require_auth = Some(value.to_string()),
            "channel_binding" => self.channel_binding = Some(value.parse()?),

            // Advanced parameters
            "target_session_attrs" => self.target_session_attrs = Some(value.parse()?),
            "load_balance_hosts" => self.load_balance_hosts = Some(value.parse()?),

            // Ignore unknown parameters (PostgreSQL behavior)
            _ => {}
        }

        Ok(())
    }

    pub fn hosts(&self) -> impl Iterator<Item = String> {
        Gen::new(|co| async move {
            let load_balance_hosts = self.load_balance_hosts.unwrap_or(LoadBalanceHosts::Disable);
            if self.host.is_empty() {
                co.yield_(format!("localhost:{}", self.port.first().copied().unwrap_or(5432)))
                    .await;
            } else if self.port.len() <= 1 {
                let port = self.port.first().copied().unwrap_or(5432);
                let mut hostnames = self.host.iter().collect::<Vec<_>>();
                if load_balance_hosts == LoadBalanceHosts::Random {
                    hostnames.shuffle(&mut rng());
                }
                for host in hostnames {
                    co.yield_(format!("{host}:{port}")).await;
                }
            } else {
                let mut hosts = self.host.iter().zip(self.port.iter()).collect::<Vec<_>>();
                if load_balance_hosts == LoadBalanceHosts::Random {
                    hosts.shuffle(&mut rng());
                }
                for (host, port) in hosts {
                    co.yield_(format!("{host}:{port}")).await;
                }
            }
        })
        .into_iter()
    }

    pub fn connect_timeout(&self) -> Duration {
        let timeout = self.connect_timeout.unwrap_or(0);
        if timeout == 0 { Duration::MAX } else { Duration::from_secs(timeout.into()) }
    }

    pub fn user(&self) -> &str {
        self.user.as_deref().unwrap_or("postgres")
    }

    pub fn database(&self) -> &str {
        self.dbname.as_deref().unwrap_or(self.user())
    }

    pub fn ssl_negotiation(&self) -> &str {
        self.sslnegotiation.as_deref().unwrap_or("postgres")
    }

    pub fn application_name(&self) -> &str {
        self.application_name.as_deref().unwrap_or("brwse")
    }
}

impl FromStr for Config {
    type Err = ParseError;

    /// Parse a connection string (either URI or key-value format)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("postgresql://") || s.starts_with("postgres://") {
            Self::parse_uri(s)
        } else {
            Self::parse_key_value(s)
        }
    }
}

/// Parse boolean values (PostgreSQL style)
fn parse_bool(s: &str) -> Result<bool, ParseError> {
    match s {
        "1" | "true" | "on" | "yes" => Ok(true),
        "0" | "false" | "off" | "no" => Ok(false),
        _ => Err(ParseError::InvalidBoolean(s.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri_basic() {
        let config = Config::from_str("postgresql://user:pass@localhost:5432/mydb").unwrap();
        assert_eq!(config.user, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert_eq!(config.host, vec!["localhost".to_string()]);
        assert_eq!(config.port, vec![5432]);
        assert_eq!(config.dbname, Some("mydb".to_string()));
    }

    #[test]
    fn test_parse_uri_with_params() {
        let config =
            Config::from_str("postgresql://user@localhost/mydb?sslmode=require&connect_timeout=10")
                .unwrap();
        assert_eq!(config.user, Some("user".to_string()));
        assert_eq!(config.host, vec!["localhost".to_string()]);
        assert_eq!(config.dbname, Some("mydb".to_string()));
        assert_eq!(config.sslmode, Some(SslMode::Require));
        assert_eq!(config.connect_timeout, Some(10));
    }

    #[test]
    fn test_parse_key_value_basic() {
        let config = Config::from_str("host=localhost port=5432 user=myuser dbname=mydb").unwrap();
        assert_eq!(config.host, vec!["localhost".to_string()]);
        assert_eq!(config.port, vec![5432]);
        assert_eq!(config.user, Some("myuser".to_string()));
        assert_eq!(config.dbname, Some("mydb".to_string()));
    }

    #[test]
    fn test_parse_key_value_quoted() {
        let config = Config::from_str("host=localhost password='my password'").unwrap();
        assert_eq!(config.host, vec!["localhost".to_string()]);
        assert_eq!(config.password, Some("my password".to_string()));
    }

    #[test]
    fn test_parse_multiple_hosts() {
        let config = Config::from_str("host=host1,host2,host3 port=5432,5433,5434").unwrap();
        assert_eq!(
            config.host,
            vec!["host1".to_string(), "host2".to_string(), "host3".to_string()]
        );
        assert_eq!(config.port, vec![5432, 5433, 5434]);
    }

    #[test]
    fn test_parse_ssl_params() {
        let config = Config::from_str("sslmode=verify-full sslrootcert=/path/to/cert").unwrap();
        assert_eq!(config.sslmode, Some(SslMode::VerifyFull));
        assert_eq!(config.sslrootcert, Some("/path/to/cert".to_string()));
    }

    #[test]
    fn test_parse_advanced_params() {
        let config =
            Config::from_str("target_session_attrs=read-write load_balance_hosts=random").unwrap();
        assert_eq!(config.target_session_attrs, Some(TargetSessionAttrs::ReadWrite));
        assert_eq!(config.load_balance_hosts, Some(LoadBalanceHosts::Random));
    }

    #[test]
    fn test_parse_uri_ipv6() {
        let config = Config::from_str("postgresql://user@[2001:db8::1234]:5432/mydb").unwrap();
        assert_eq!(config.user, Some("user".to_string()));
        assert_eq!(config.host, vec!["2001:db8::1234".to_string()]);
        assert_eq!(config.port, vec![5432]);
        assert_eq!(config.dbname, Some("mydb".to_string()));
    }

    #[test]
    fn test_parse_escaped_values() {
        let config = Config::from_str(r"password=my\'password host=localhost").unwrap();
        assert_eq!(config.password, Some("my'password".to_string()));
        assert_eq!(config.host, vec!["localhost".to_string()]);
    }

    #[test]
    fn test_from_str_trait() {
        // Test using the FromStr trait
        let config: Config = "postgresql://user@localhost/mydb".parse().unwrap();
        assert_eq!(config.user, Some("user".to_string()));
        assert_eq!(config.host, vec!["localhost".to_string()]);
        assert_eq!(config.dbname, Some("mydb".to_string()));
    }

    #[test]
    fn test_hosts_default() {
        let config = Config::default();
        let hosts: Vec<_> = config.hosts().collect();
        assert_eq!(hosts, vec!["localhost:5432"]);
    }

    #[test]
    fn test_hosts_single_host() {
        let config = Config {
            host: vec!["db.example.com".to_string()],
            port: vec![6543],
            ..Default::default()
        };
        let hosts: Vec<_> = config.hosts().collect();
        assert_eq!(hosts, vec!["db.example.com:6543"]);
    }

    #[test]
    fn test_hosts_multiple_hosts_and_ports() {
        let config = Config {
            host: vec!["host1".to_string(), "host2".to_string()],
            port: vec![1111, 2222],
            ..Default::default()
        };
        let hosts: Vec<_> = config.hosts().collect();
        // With multiple ports, hosts() zips host and port
        assert_eq!(hosts, vec!["host1:1111", "host2:2222"]);
    }

    #[test]
    fn test_hosts_more_ports_than_hosts() {
        let config = Config {
            host: vec!["host1".to_string()],
            port: vec![1111, 2222, 3333],
            ..Default::default()
        };
        let hosts: Vec<_> = config.hosts().collect();
        // With more ports than hosts, zipping will only yield as many as the shortest
        assert_eq!(hosts, vec!["host1:1111"]);
    }

    #[test]
    fn test_hosts_more_hosts_than_ports() {
        let config = Config {
            host: vec!["host1".to_string(), "host2".to_string()],
            port: vec![9999],
            ..Default::default()
        };
        let hosts: Vec<_> = config.hosts().collect();
        // With one port, all hosts use that port
        assert_eq!(hosts, vec!["host1:9999", "host2:9999"]);
    }
}
