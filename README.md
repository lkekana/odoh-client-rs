# odoh-client-rs

[odoh-client-rs] is a CLI Rust client that can be used to access resolvers running the [Oblivious DNS over HTTPS (ODoH) protocol draft-06]. It is built using the [odoh-rs] library. It is mainly intended for testing as it can only send one request at a time. 

[odoh-client-rs]: https://github.com/cloudflare/odoh-client-rs/
[Oblivious DNS over HTTPS (ODoH) protocol draft-06]: https://tools.ietf.org/html/draft-pauly-dprive-oblivious-doh-06
[odoh-rs]: https://github.com/cloudflare/odoh-rs/

# Example usage

The proxy and resolver are configured by default using the file specified by the `-c` flag, e.g., `-c config.toml`. The default configuration can be found at `tests/config.toml`. You can also specify the proxy and resolver using the `-p` and `-r` flags, respectively.

By default, it uses https://odoh.cloudflare-dns.com, i.e., 1.1.1.1, as the target resolver, and a well known endpoint to retrieve the configs via `GET` requests.

```bash
$ cargo run -- example.com
$ cargo run -- example.com A
$ cargo run -- example.com AAAA
$ cargo run -- example.com AAAA -c config.toml
$ cargo run -- example.com AAAA -p https://odoh.cloudflare-dns.com -r https://odoh.cloudflare-dns.com
$ cargo run -- example.com AAAA -c config.toml
```
