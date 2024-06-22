# fluent-uri

A full-featured URI handling library compliant with [RFC 3986]. It is:

- **Fast:** Zero-copy parsing. Benchmarked to be highly performant.[^bench-res]
- **Easy:** Carefully designed and documented APIs. Handy percent-encoding utilities.
- **Correct:** Forbids unsafe code. Extensively fuzz-tested against other implementations.

[![crates.io](https://img.shields.io/crates/v/fluent-uri.svg)](https://crates.io/crates/fluent-uri)
[![build](https://img.shields.io/github/actions/workflow/status/yescallop/fluent-uri-rs/ci.yml
)](https://github.com/yescallop/fluent-uri-rs/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/fluent-uri.svg)](/LICENSE)

[Documentation](https://docs.rs/fluent-uri) | [Discussions](https://github.com/yescallop/fluent-uri-rs/discussions)

[RFC 3986]: https://datatracker.ietf.org/doc/html/rfc3986/
[^bench-res]: In [a benchmark](https://github.com/yescallop/fluent-uri-rs/blob/main/bench/benches/bench.rs)
    on an Intel Core i5-11300H processor, `fluent-uri` parsed a URI
    in 49ns compared to 89ns for `iref` and 135ns for `iri-string`.

## Examples

- Parse and extract components zero-copy from a URI reference:

    ```rust
    const SCHEME_FOO: &Scheme = Scheme::new_or_panic("foo");

    let uri = Uri::parse("foo://user@example.com:8042/over/there?name=ferret#nose")?;

    assert_eq!(uri.scheme().unwrap(), SCHEME_FOO);

    let auth = uri.authority().unwrap();
    assert_eq!(auth.as_str(), "user@example.com:8042");
    assert_eq!(auth.userinfo().unwrap(), "user");
    assert_eq!(auth.host(), "example.com");
    assert!(matches!(auth.host_parsed(), Host::RegName(name) if name == "example.com"));
    assert_eq!(auth.port().unwrap(), "8042");
    assert_eq!(auth.port_to_u16(), Ok(Some(8042)));

    assert_eq!(uri.path(), "/over/there");
    assert_eq!(uri.query().unwrap(), "name=ferret");
    assert_eq!(uri.fragment().unwrap(), "nose");
    ```

- Build a URI reference using the builder pattern:

    ```rust
    const SCHEME_FOO: &Scheme = Scheme::new_or_panic("foo");

    let uri = Uri::builder()
        .scheme(SCHEME_FOO)
        .authority(|b| {
            b.userinfo(EStr::new_or_panic("user"))
                .host(EStr::new_or_panic("example.com"))
                .port(8042)
        })
        .path(EStr::new_or_panic("/over/there"))
        .query(EStr::new_or_panic("name=ferret"))
        .fragment(EStr::new_or_panic("nose"))
        .build()
        .unwrap();

    assert_eq!(
        uri.as_str(),
        "foo://user@example.com:8042/over/there?name=ferret#nose"
    );
    ```

- Resolve a URI reference against a base URI:

    ```rust
    let base = Uri::parse("http://example.com/foo/bar")?;

    assert_eq!(Uri::parse("baz")?.resolve_against(&base)?, "http://example.com/foo/baz");
    assert_eq!(Uri::parse("../baz")?.resolve_against(&base)?, "http://example.com/baz");
    assert_eq!(Uri::parse("?baz")?.resolve_against(&base)?, "http://example.com/foo/bar?baz");
    ```

- Normalize a URI reference:

    ```rust
    let uri = Uri::parse("eXAMPLE://a/./b/../b/%63/%7bfoo%7d")?;
    assert_eq!(uri.normalize(), "example://a/b/c/%7Bfoo%7D");
    ```

- `EStr` (Percent-encoded string slices):

    All components in a URI that may be percent-encoded are parsed as `EStr`s,
    which allows easy splitting and decoding:

    ```rust
    let s = "?name=%E5%BC%A0%E4%B8%89&speech=%C2%A1Ol%C3%A9%21";
    let query = Uri::parse(s).unwrap().query().unwrap();
    let map: HashMap<_, _> = query
        .split('&')
        .map(|s| s.split_once('=').unwrap_or((s, EStr::EMPTY)))
        .map(|(k, v)| (k.decode().into_string_lossy(), v.decode().into_string_lossy()))
        .collect();
    assert_eq!(map["name"], "张三");
    assert_eq!(map["speech"], "¡Olé!");
    ```

- `EString` (A percent-encoded, growable string):

    You can encode key-value pairs to a query string and use it to build a `Uri`:

    ```rust
    let pairs = [("name", "张三"), ("speech", "¡Olé!")];
    let mut buf = EString::<Query>::new();
    for (k, v) in pairs {
        if !buf.is_empty() {
            buf.push_byte(b'&');
        }
        buf.encode::<Data>(k);
        buf.push_byte(b'=');
        buf.encode::<Data>(v);
    }

    assert_eq!(buf, "name=%E5%BC%A0%E4%B8%89&speech=%C2%A1Ol%C3%A9%21");

    let uri = Uri::builder()
        .path(EStr::EMPTY)
        .query(&buf)
        .build();
    assert_eq!(uri.as_str(), "?name=%E5%BC%A0%E4%B8%89&speech=%C2%A1Ol%C3%A9%21");
    ```
