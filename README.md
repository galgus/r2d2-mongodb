[![crates.io](https://img.shields.io/crates/v/r2d2-mongodb.svg)](https://crates.io/crates/r2d2-mongodb)
[![docs.rs](https://docs.rs/r2d2-mongodb/badge.svg)](https://docs.rs/r2d2-mongodb)

# r2d2-mongodb

A MongoDB adaptor for r2d2 connection pool.

## Documentation

[In progress...](https://docs.rs/r2d2-mongodb)

## Example usage

Start mongodb:

```shell
$ docker run --rm -p 27017:27017 -e MONGO_INITDB_ROOT_USERNAME=root -e MONGO_INITDB_ROOT_PASSWORD=password -e MONGO_INITDB_DATABASE=mydb mongo:latest
```

```rust
extern crate r2d2;
extern crate r2d2_mongodb;

use r2d2::Pool;
use r2d2_mongodb::{ConnectionOptions, MongodbConnectionManager};

fn main () {
    let manager = MongodbConnectionManager::new(
        ConnectionOptions::builder()
            .with_host("localhost", 27017)
            .with_db("mydb")
            .with_auth("root", "password")
            .build()
    );

    let pool = Pool::builder()
        .max_size(16)
        .build(manager)
        .unwrap();

    // ...
}
```
