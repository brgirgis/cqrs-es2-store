# cqrs-es2-store

**Sync implementation of the cqrs-es2 store.**

[![Publish](https://github.com/brgirgis/cqrs-es2-store/actions/workflows/crates-io.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2-store/actions/workflows/crates-io.yml)
[![Test](https://github.com/brgirgis/cqrs-es2-store/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/brgirgis/cqrs-es2-store/actions/workflows/rust-ci.yml)
[![Latest version](https://img.shields.io/crates/v/cqrs-es2-store)](https://crates.io/crates/cqrs-es2-store)
[![docs](https://img.shields.io/badge/API-docs-blue.svg)](https://docs.rs/cqrs-es2-store)
![License](https://img.shields.io/crates/l/cqrs-es2-store.svg)

---

Provides sync interfaces to different database implementations for the CQRS system store.

## Design

The main components of this library are:

- `IEventDispatcher` - an interface for sync events listeners
- `IEventStore` - an interface for sync event stores
- `IQueryStore` - an interface for sync query stores

## Features

- `with-postgres` - sync Postgres store
- `with-all-sync` - all sync drivers (default)

# Installation

To use this library in a sync application, add the following to
your dependency section in the project's `Cargo.toml`:

````toml
[dependencies]
# logging
log = { version = "^0.4", features = [
  "max_level_debug",
  "release_max_level_warn",
] }
fern = "^0.5"

# serialization
serde = { version = "^1.0.127", features = ["derive"] }
serde_json = "^1.0.66"

# CQRS framework
cqrs-es2 = { version = "*"}

# Sync postgres store implementation
cqrs-es2-store = { version = "*", default-features = false, features = [
   "with-postgres",
 ] }

 # postgres driver
 postgres = { version = "^0.19.1", features = ["with-serde_json-1"] }
 ```

 # Usage

 A full sync store example application is available [here](https://github.com/brgirgis/cqrs-es2-store/tree/master/examples/restful).

## Change Log

A complete history of the change log can be found [here](https://github.com/brgirgis/cqrs-es2-store/blob/master/ChangeLog.md)

## TODO

An up-to-date list of development aspirations can be found [here](https://github.com/brgirgis/cqrs-es2-store/blob/master/TODO.md)
````
