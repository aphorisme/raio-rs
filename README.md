# raio

[![Crates.io][crates-badge]][crates-url]
[![Docs.io][docs-badge]][docs-url]


[docs-badge]: https://docs.rs/raio/badge.svg
[docs-url]: https://docs.rs/raio

[crates-badge]: https://img.shields.io/crates/v/raio.svg
[crates-url]: https://crates.io/crates/raio

An opinionated async [Bolt protocol](https://7687.org/#bolt) client implementation written in Rust 🦀.

| Supports Bolt Versions: | 4.1, 4.0 |
| :----- | :---- |

⚠️ This is a rewrite of `raio-0.1.0`. There is no migration possible.

⚠️ This package has yet to proof to be largely bug free.

⚠️ Be aware, that this package is based upon the specification and not upon
another driver. It is part of a bigger project, hence heavy changes might
appear in the future and it is opinionated.

The [packs](https://github.com/aphorisme/packs-rs) package drives the PackStream part.

## Usage

The high-level entry-point is `Client` which gives the functionality to run queries
and open transactions. It comes with a connection pool the handle asynchronous
actions.

````rust
use raio::client::{Client, ClientConfig};
use raio::client::auth::Basic;
use raio::client::error::ClientError;
use raio::messaging::query::Query;

#[async_std::test]
pub async fn auto_commit_simple() -> Result<(), ClientError> {
    let auth = Basic::new("neo4j", "mastertest");
    let client =
        Client::create(
            "localhost:7687",
            auth,
            ClientConfig::default("raio-rs-test", "0.2.0"));

    let mut query = Query::new("RETURN $x + 1 as x, $y as y, $b as b");
    query.param("x", 1);
    query.param("y", Some(String::from("Hello")));
    query.param("b", true);

    let result =
        client
            .query(&query)
            .await
            .expect("Error while querying.");

    let first =
        result
            .records()
            .first()
            .expect("Expected at least one result");

    assert_eq!(first.get_field_typed("x"), Some(&2));
    assert_eq!(first.get_field_typed("y"), Some(&String::from("Hello")));
    assert_eq!(first.get_field_typed("b"), Some(&true));

    Ok(())
}
````

## Contribution

You are welcome to contribute! This package is still in its very early days,
so there will be bugs to be discovered and fixed, testing is quite short,
documentation is lacking. Since I will use this package in a bigger project,
progress will be made eventually, but any help is appreciated to turn this into
a usable and performant bolt driver.