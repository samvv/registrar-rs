[![docs.rs](https://img.shields.io/docsrs/openprovider/latest?style=flat-square)](https://docs.rs/openprovider)

[OpenProvider](https://openprovider.com) is a domain registrar based in the Netherlands.
The service features a public API that anyone can make use of.

This crate implements a subset of that API in Rust. With it, you can query, filter and
manipulate DNS records.

Unforunately, this crate is not complete yet. Many more APIs, such as SSL certificates, have
yet to be implemented. You are invited to try out the API and contribute to the project [back
on GitHub](https://github.com/samvv/openprovider-rs).

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

