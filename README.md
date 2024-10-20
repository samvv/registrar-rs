Registrar Clients in Rust
=========================

[![tests](https://github.com/samvv/registrar-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/samvv/registrar-rs/actions/workflows/rust.yml)

This repository contains clients for different registrars. Using these
libraries, you can query and manipulate your domains as if they are part of one
big domain pool.

Some cool things you can do with this library:

 - Build a web interface so that all your domains are manage in one place
 - Build and run a custom **dynamic DNS** client, such as [dnsmaster].
 - Create a CLI tool to manage your domains

[dnsmaster]: https://github.com/samvv/dnsmaster

> [!WARNING]
>
> Not everything has yet been implementaed in these libraries. You are invited
> to file bugs or create a pull request.

```rust
use registrar::Error;

let mut client = openprovider::Client::default();

loop {
    match client.list_dns_records("example.com").await {
        Ok(info) => break info,
        Err(Error::AuthenticationFailed) => {
            let token = client.login("bob", "123456789").await?;
            client.set_token(token);
        },
        Err(error) => panic!("Failed to fetch DNS zone info: {}", error),
    }
}
```
