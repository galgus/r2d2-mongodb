//! # r2d2-mongodb
//! A MongoDB adaptor for r2d2 connection pool.
//! # Example
//! ```rust
//! extern crate r2d2;
//! extern crate r2d2_mongodb;
//!
//! use r2d2::Pool;
//! use r2d2_mongodb::{ConnectionOptions, MongodbConnectionManager};
//!
//! fn main () {
//!     let manager = MongodbConnectionManager::new(
//!         ConnectionOptions::builder()
//!             .with_host("localhost", 27017)
//!             .with_ssl(
//!                 Some("path/to/ca.crt"),
//!                 "path/to/client.crt",
//!                 "path/to/client.key",
//!                 true
//!             )
//!             .with_db("mydb")
//!             .with_auth("root", "password")
//!             .build()
//!     );
//!
//!     // let pool = Pool::builder()
//!     //     .max_size(16)
//!     //     .build(manager)
//!     //     .unwrap();
//!
//!     // ...
//! }
//! ```

pub extern crate mongodb;
pub extern crate r2d2;
extern crate rand;

use mongodb::connstring::parse;
use mongodb::db::{Database, ThreadedDatabase};
use mongodb::{Client, Error, ThreadedClient};
use r2d2::ManageConnection;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone)]
pub struct Host {
    /// Address of the MongoDB server
    ///
    /// Default: `"localhost"`
    pub hostname: String,
    /// Port on which to connect
    ///
    /// Default: `27017`
    pub port: u16,
}

impl Default for Host {
    fn default() -> Host {
        Host {
            hostname: "localhost".to_string(),
            port: 27017,
        }
    }
}

#[derive(Clone)]
pub struct Auth {
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
}

#[derive(Clone)]
pub enum SSLKind {
    Authenticated,
    Unauthenticated,
}

#[derive(Clone)]
pub struct SSLConfig {
    pub ca_file: Option<String>,
    pub certificate_file: Option<String>,
    pub key_file: Option<String>,
    pub verify_peer: bool,
    pub kind: SSLKind,
}

/// Options with which the connections to MongoDB will be created
///
/// To authenticate the connection you have to provide both a `username` and `password`.
#[derive(Clone)]
pub struct ConnectionOptions {
    /// List of hosts
    ///
    /// Default: `vec![]`
    pub hosts: Vec<Host>,
    /// Name of the database to connect to
    ///
    /// Default: `"admin"`
    pub db: String,
    /// Authentication options
    ///
    /// Default: `None`
    pub auth: Option<Auth>,
    /// SSL options
    ///
    /// Default: `None`
    pub ssl: Option<SSLConfig>,
}

impl Default for ConnectionOptions {
    fn default() -> ConnectionOptions {
        ConnectionOptions {
            hosts: vec![],
            db: "admin".to_string(),
            auth: None,
            ssl: None,
        }
    }
}

impl ConnectionOptions {
    pub fn builder() -> ConnectionOptionsBuilder {
        ConnectionOptionsBuilder(ConnectionOptions::default())
    }
}

/// Builder for `ConnectionOptions`
pub struct ConnectionOptionsBuilder(ConnectionOptions);

impl ConnectionOptionsBuilder {
    pub fn with_host(&mut self, hostname: &str, port: u16) -> &mut ConnectionOptionsBuilder {
        self.0.hosts.push(Host {
            hostname: hostname.to_string(),
            port,
        });
        self
    }

    pub fn with_db(&mut self, db: &str) -> &mut ConnectionOptionsBuilder {
        self.0.db = db.to_string();
        self
    }

    pub fn with_auth(&mut self, username: &str, password: &str) -> &mut ConnectionOptionsBuilder {
        self.0.auth = Some(Auth {
            username: username.to_string(),
            password: password.to_string(),
        });
        self
    }

    pub fn with_ssl(
        &mut self,
        ca_file: Option<&str>,
        certificate_file: &str,
        key_file: &str,
        verify_peer: bool,
    ) -> &mut ConnectionOptionsBuilder {
        self.0.ssl = Some(SSLConfig {
            ca_file: match ca_file {
                Some(ca_file) => Some(ca_file.to_string()),
                None => None
            },
            certificate_file: Some(certificate_file.to_string()),
            key_file: Some(key_file.to_string()),
            verify_peer,
            kind: SSLKind::Authenticated,
        });
        self
    }

    pub fn with_unauthenticated_ssl(
        &mut self,
        ca_file: Option<&str>,
        verify_peer: bool,
    ) -> &mut ConnectionOptionsBuilder {
        self.0.ssl = Some(SSLConfig {
            ca_file: match ca_file {
                Some(ca_file) => Some(ca_file.to_string()),
                None => None
            },
            certificate_file: None,
            key_file: None,
            verify_peer,
            kind: SSLKind::Unauthenticated,
        });
        self
    }

    pub fn build(&self) -> ConnectionOptions {
        self.0.clone()
    }
}

/// Struct for managing a pool of MongoDB connections
pub struct MongodbConnectionManager {
    options: ConnectionOptions,
}

impl MongodbConnectionManager {
    pub fn new(options: ConnectionOptions) -> MongodbConnectionManager {
        MongodbConnectionManager { options }
    }

    pub fn new_with_uri(uri: &str) -> Result<MongodbConnectionManager, Error> {
        let cs = parse(uri)?;
        let mut options_builder = ConnectionOptions::builder();

        if let Some(db) = cs.database {
            options_builder.with_db(&db);
        }

        if let (Some(user), Some(password)) = (cs.user, cs.password) {
            options_builder.with_auth(&user, &password);
        }

        for h in cs.hosts {
            options_builder.with_host(&h.host_name, h.port);
        }

        let options = options_builder.build();
        Ok(MongodbConnectionManager { options })
    }
}

impl ManageConnection for MongodbConnectionManager {
    type Connection = Database;
    type Error = Error;

    fn connect(&self) -> Result<Database, Error> {
        let mut rng = thread_rng();
        let host = self
            .options
            .hosts
            .as_slice()
            .choose(&mut rng)
            .ok_or(Error::ArgumentError("No host provided".into()))?;

        let client = match &self.options.ssl {
            Some(ssl_options) => {
                let client_options = match ssl_options.kind {
                    SSLKind::Authenticated => mongodb::ClientOptions::with_ssl(
                        match &ssl_options.ca_file {
                            Some(ca_file) => Some(ca_file.as_str()),
                            None => None
                        },
                        match &ssl_options.certificate_file {
                            Some(certificate_file) => certificate_file.as_str(),
                            None => panic!("certificate_file is required with SSLKind::Authenticated")
                        },
                        match &ssl_options.key_file {
                            Some(key_file) => key_file.as_str(),
                            None => panic!("key_file is required with SSLKind::Authenticated")
                        },
                        ssl_options.verify_peer,
                    ),
                    SSLKind::Unauthenticated => mongodb::ClientOptions::with_unauthenticated_ssl(
                        match &ssl_options.ca_file {
                            Some(ca_file) => Some(ca_file.as_str()),
                            None => None
                        },
                        ssl_options.verify_peer,
                    ),
                };
                Client::connect_with_options(&host.hostname, host.port, client_options)?
            }
            None => Client::connect(&host.hostname, host.port)?,
        };
        let db = client.db(&self.options.db);

        if let Some(ref auth) = self.options.auth {
            db.auth(&auth.username, &auth.password)?;
        }

        Ok(db)
    }

    fn is_valid(&self, db: &mut Database) -> Result<(), Error> {
        db.version()?;
        Ok(())
    }

    fn has_broken(&self, _db: &mut Database) -> bool {
        false
    }
}
