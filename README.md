[![crates.io](https://img.shields.io/crates/v/r2d2-mongodb.svg)](https://crates.io/crates/r2d2-mongodb)
[![docs.rs](https://docs.rs/r2d2-mongodb/badge.svg)](https://docs.rs/r2d2-mongodb)

# r2d2-mongodb

A MongoDB adaptor for r2d2 connection pool.

## Documentation

[In progress...](https://docs.rs/r2d2-mongodb)

## Example usage

```rust
extern crate r2d2;
extern crate r2d2_mongodb;

use r2d2::Pool;
use r2d2_mongodb::{ConnectionOptionsBuilder, MongodbConnectionManager};

fn main () {
    let manager = MongodbConnectionManager::new(
        ConnectionOptionsBuilder::new()
            .with_host("localhost")
            .with_port(27017)
            .with_db("admin")
            .with_username("root")
            .with_password("password")
            .build()
    );

    let pool = Pool::builder()
        .max_size(64)
        .build(manager)
        .unwrap();

    //...
}
```
