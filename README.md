OpenProvider.nl in Rust
=======================

This is an implementation of a spec-compliant HTTP-based API client for the
OpenProvider.nl domain reseller.

**⚠️ Right now only a small subset of the OpenProvider API is supported, but 
with most groundwork being done it should be easy to integrate whatever
endpoint you need.**

## Usage

Basic setup is as follows:

```rs
let client = openprovider::Builder::new()
  // more configuration option will go here in the future ...
  .build();

let token = client.login("bob.ross@gmail.com", "averygoodpassword").await.unwrap();

client.set_token(token);
```

You can now use the client to make authorized requests to the OpenProvider API,
like so:

```rs
let zones = client.get_zone("vervaeck.net").await.unwrap();

eprintln!("Zone vervaeck.net created at {}", zone.creation_date);
```

## API

Currently, only a limited set of API endpoints are implemented.

Each method returns a future that you must `await` on before finally getting
the result with the data or error code.

### `client.list_zones()`

List all DNS zones avaialable to the currently authorized user.

### `client.get_zone(zone_name)`

Get more information about a specific DNS zone, excluding all of its records.
This can be found using `list_records()`.

### `client.list_records(zone_name)`

List all records belonging to a specific domain

### `client.set_record(zone_name, old, new)`

Update a given record with new attributes.

Due to the way the OpenProvider API works, you must supply the old record as
well.

## License

This piece of software is licensed under the MIT licence.

Buying the author a coffee is always much appreciated.

