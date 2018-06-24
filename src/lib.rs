pub extern crate mongodb;
pub extern crate r2d2;

use r2d2::ManageConnection;
use mongodb::{ThreadedClient, Client, Error};
use mongodb::db::{ThreadedDatabase, Database};

#[derive(Clone)]
pub struct ConnectionOptions {
    host: String,
    port: u16,
    db: String,
    username: Option<String>,
    password: Option<String>,
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

pub struct MongodbConnectionManagerBuilder {
    options: ConnectionOptions,
}

impl MongodbConnectionManagerBuilder {
    pub fn new() -> MongodbConnectionManagerBuilder {
        MongodbConnectionManagerBuilder{
            options: ConnectionOptions::default()
        }
    }

    pub fn with_host<'a>(&'a mut self, host: &str) -> &'a mut MongodbConnectionManagerBuilder {
        self.options.host = host.to_string();
        self
    }

    pub fn with_port<'a>(&'a mut self, port: u16) -> &'a mut MongodbConnectionManagerBuilder {
        self.options.port = port;
        self
    }

    pub fn with_db<'a>(&'a mut self, db: &str) -> &'a mut MongodbConnectionManagerBuilder {
        self.options.db = db.to_string();
        self
    }

    pub fn with_username<'a>(&'a mut self, username: &str) -> &'a mut MongodbConnectionManagerBuilder {
        self.options.username = Some(username.to_string());
        self
    }

    pub fn with_password<'a>(&'a mut self, password: &str) -> &'a mut MongodbConnectionManagerBuilder {
        self.options.password = Some(password.to_string());
        self
    }

    pub fn build(&mut self) -> MongodbConnectionManager {
        MongodbConnectionManager{
            options: self.options.clone()
        }
    }
}

pub struct MongodbConnectionManager {
    options: ConnectionOptions,
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
