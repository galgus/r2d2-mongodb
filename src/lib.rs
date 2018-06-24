//! # r2d2-mongodb
//! A MongoDB adaptor for r2d2 connection pool.
//! # Example
//! ```rust
//! extern crate r2d2;
//! extern crate r2d2_mongodb;
//!
//! use r2d2::Pool;
//! use r2d2_mongodb::{ConnectionOptionsBuilder, MongodbConnectionManager};
//!
//! fn main () {
//!     let manager = MongodbConnectionManager::new(
//!         ConnectionOptionsBuilder::new()
//!             .with_host("localhost")
//!             .with_port(27017)
//!             .with_db("admin")
//!             .with_username("root")
//!             .with_password("password")
//!             .build()
//!     );
//!
//!     let pool = Pool::builder()
//!         .max_size(64)
//!         .build(manager)
//!         .unwrap();
//!
//!     //...
//! }
//! ```

pub extern crate mongodb;
pub extern crate r2d2;

use r2d2::ManageConnection;
use mongodb::{ThreadedClient, Client, Error};
use mongodb::db::{ThreadedDatabase, Database};

/// Options with which the connections to MongoDB will be created
///
/// To authenticate the connection you have to provide both a `username` and `password`.
#[derive(Clone)]
pub struct ConnectionOptions {
    /// Address of the MongoDB server
    ///
    /// Default: `"localhost"`
    pub host: String,
    /// Port on which to connect
    ///
    /// Default: `27017`
    pub port: u16,
    /// Name of the database to connect to
    ///
    /// Default: `"admin"`
    pub db: String,
    /// Username for authentication
    ///
    /// Default: `None`
    pub username: Option<String>,
    /// Password for authentication
    ///
    /// Default: `None`
    pub password: Option<String>,
}

impl Default for ConnectionOptions {
    fn default() -> ConnectionOptions {
        ConnectionOptions {
            host: "localhost".to_string(),
            port: 27017,
            db: "admin".to_string(),
            username: None,
            password: None,
        }
    }
}

/// Builder for `ConnectionOptions`
pub struct ConnectionOptionsBuilder(ConnectionOptions);

impl ConnectionOptionsBuilder {
    pub fn new() -> ConnectionOptionsBuilder {
        ConnectionOptionsBuilder(ConnectionOptions::default())
    }

    pub fn with_host<'a>(&'a mut self, host: &str) -> &'a mut ConnectionOptionsBuilder {
        self.0.host = host.to_string();
        self
    }

    pub fn with_port<'a>(&'a mut self, port: u16) -> &'a mut ConnectionOptionsBuilder {
        self.0.port = port;
        self
    }

    pub fn with_db<'a>(&'a mut self, db: &str) -> &'a mut ConnectionOptionsBuilder {
        self.0.db = db.to_string();
        self
    }

    pub fn with_username<'a>(&'a mut self, username: &str) -> &'a mut ConnectionOptionsBuilder {
        self.0.username = Some(username.to_string());
        self
    }

    pub fn with_password<'a>(&'a mut self, password: &str) -> &'a mut ConnectionOptionsBuilder {
        self.0.password = Some(password.to_string());
        self
    }

    pub fn build(&mut self) -> ConnectionOptions {
        self.0.clone()
    }
}

/// Struct for managing a pool of MongoDB connections
pub struct MongodbConnectionManager {
    options: ConnectionOptions,
}

impl MongodbConnectionManager {
    pub fn new(options: ConnectionOptions) -> MongodbConnectionManager {
        MongodbConnectionManager {
            options
        }
    }
}

impl ManageConnection for MongodbConnectionManager {
    type Connection = Database;
    type Error = Error;

    fn connect(&self) -> Result<Database, Error> {
        let client = Client::connect(&self.options.host, self.options.port)?;
        let db = client.db(&self.options.db);

        if let (&Some(ref username), &Some(ref password)) = (&self.options.username, &self.options.password) {
            db.auth(&username, &password)?;
        }

        Ok(db)
    }

    fn is_valid(&self, db: &mut Database) -> Result<(), Error> {
        db.version().map(|_| ())
    }

    fn has_broken(&self, _db: &mut Database) -> bool {
        false
    }
}
