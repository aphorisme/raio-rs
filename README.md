# raio
An async  [Bolt protocol](https://7687.org/#bolt) client implementation written in Rust 🦀.

| Supports Bolt Version: | 4.1 |
| :----- | :---- |

⚠️ This is a rewrite of `raio-0.1.0`. There is no migration possible.

⚠️ This package has yet to proof to be largely bug free.



The [packs](https://github.com/aphorisme/packs-rs) package drives the PackStream part.

# Overview


# Technical Overview

There is `Connection` which provides the low level runtime building 
block, but does not implement any client logic. It provides the async
functionality to connect to a bolt server, send requests and read
responses. Among these low level functionalities, there is `Manager`
and `Pool` as well. This level is based upon `deadpool` to provide
a pool of connections for async handling.

The high level entry point is `Client`. This does use a connection
pool and the low level `Connection` to implement all client logic,
e.g. transactions, queries, etc.

