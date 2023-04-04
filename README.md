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
let client = openprovider::Client::default();

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



## License

This piece of software is licensed under the MIT licence.

Buying the author a coffee is always much appreciated.

