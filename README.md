# Overview

This package is a **Work-In-Progress** bolt driver for neo4j. Since I need a way
to interact with neo4j from rust, I've written this driver and as such, it is as
minimal as possible to make it usable. It can open a connection to neo4j and run
cypher queries.

The plan is to grow this package into some mature bolt driver by implementing more
features as I need them in my other projects.

If you want to contribute, you can! Either by requesting features or even better: by 
providing pull requests (don't forget to rebase). Design is discussable.

## TODO

☐ Documentation: general.

☐ Inline documentation and tests.

☐ Integration Tests, maybe using `boltkit`.
  
☐ Version 3 Bolt types: `DateTime`, etc.

☐ More advanced synchronous client: `Commit`, `Rollback`, `Transaction`.

☐ Split client in `Driver` and multi-thread-safe `Session` to allow several `Sessions` in parallel.

☐ Asynchronous client?

# Framework Ideas

The package comes in two parts: a mere `packing` part which is responsible
for all bolt protocol types and their encoding and decoding. 

The second part, `net`, is responsible for the networking and provides a synchronous
bolt client `raio::net::sync::Client` which can be used to connect to a neo4j graph
database and run cypher statements against it.

## The `packing` Part

The `packing` itself is divided into different levels of abstraction. As a 
hole it is meant to provide two functions

```rust
fn pack_to<T: BoltWrite>(self, buf: &mut T) -> Result<usize, PackingError>;
```

which packs a hole value into a string of bytes and 

```rust
fn unpack_from<T: BoltRead>(buf: &mut T) -> Result<Self, UnpackingError>;
```

which reads such values back into Rust. 

Hence **Packing**, with its traits `Packable` and `Unpackable`, is the the top-level of all encoding. 
It does not only encode 
strings as utf8 encoded byte strings or an `i64` big endian, but packs 
values as bolt values, i.e. with an appropriated header such that a server
can understand and `unpack` the value as a string or `i64`. 

The bolt protocol defines different kinds of types which are packed in different
ways. There are

 - *Mere Marker Types*, which consist only of a marker byte like `True`, `False` and `Null`. 
 - *Tiny Sized Types*, which consist of a combined size and marker byte as their header,
 like `TinyString` and `TinyMap`,
 - *Sized Types*, which consist of a marker byte and a dedicated encoded size afterwards,
 like `String8` and `TinyList16`,
 - *Fixed Types*, which consist of a marker byte and are of fixed size like `Float64`, `Int8`.
 
Each of these kinds have their own trait. 

The lowest level of encoding is `BoltWriteable` and `BoltReadable` which gives
a unified interface for the mere encoding and decoding of base types. For 
example, it defines that all the integer types are written in a big endian way,
that strings are read and written as utf8 encoded, etc.

# Protocol Versions

`raio-rs` uses bolt version 3. 

`Init`  --> `Hello`

`Run <cypher> <params>` --> `Run <cypher> <params> <meta>`
