# odoh-client-rs

[odoh-client-rs] is a CLI Rust client that can be used to access resolvers running the [Oblivious DNS over HTTPS (ODoH) protocol draft-06]. It is built using the [odoh-rs] library. It is mainly intended for testing as it can only send one request at a time. 

[odoh-client-rs]: https://github.com/lkekana/odoh-client-rs/
[Oblivious DNS over HTTPS (ODoH) protocol draft-06]: https://tools.ietf.org/html/draft-pauly-dprive-oblivious-doh-06
[odoh-rs]: https://github.com/cloudflare/odoh-rs/

# Example usage

The proxy and target resolver are configured by default using the file specified by the `-c` flag, e.g., `-c config.toml`. The default configuration can be found at `tests/config.toml`. You can also specify the proxy and target resolver using the `-p` and `-t` flags, respectively.

By default, it uses https://odoh.cloudflare-dns.com, i.e., 1.1.1.1, as the target resolver, and a well known endpoint to retrieve the configs via `GET` requests.

```bash
$ cargo run -- example.com
$ cargo run -- example.com A
$ cargo run -- example.com AAAA
$ cargo run -- -c config.toml example.com AAAA
$ cargo run -- -t https://odoh.cloudflare-dns.com example.com AAAA
```