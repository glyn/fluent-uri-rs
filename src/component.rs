//! Components of URI reference.

use crate::{
    encoding::{
        encoder::{Port, RegName, Userinfo},
        table, EStr,
    },
    internal::{AuthMeta, HostMeta},
};
use core::num::ParseIntError;
use ref_cast::{ref_cast_custom, RefCastCustom};

#[cfg(feature = "net")]
use crate::net::{Ipv4Addr, Ipv6Addr};

#[cfg(all(feature = "net", feature = "std"))]
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

/// The [scheme] component of URI reference.
///
/// [scheme]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.1
///
/// # Comparison
///
/// `Scheme`s are compared case-insensitively. You should do a case-insensitive
/// comparison if the scheme specification allows both letter cases in the scheme name.
///
/// # Examples
///
/// ```
/// use fluent_uri::{component::Scheme, UriRef};
///
/// const SCHEME_HTTP: &Scheme = Scheme::new_or_panic("http");
///
/// let uri_ref = UriRef::parse("HTTP://EXAMPLE.COM/")?;
/// let scheme = uri_ref.scheme().unwrap();
///
/// // Case-insensitive comparison.
/// assert_eq!(scheme, SCHEME_HTTP);
/// // Case-sensitive comparison.
/// assert_eq!(scheme.as_str(), "HTTP");
/// # Ok::<_, fluent_uri::error::ParseError>(())
/// ```
#[derive(RefCastCustom)]
#[repr(transparent)]
pub struct Scheme {
    inner: str,
}

impl Scheme {
    #[ref_cast_custom]
    #[inline]
    pub(crate) const fn new_validated(scheme: &str) -> &Scheme;

    /// Converts a string slice to `&Scheme`.
    ///
    /// # Panics
    ///
    /// Panics if the string is not a valid scheme name according to
    /// [Section 3.1 of RFC 3986][scheme]. For a non-panicking variant,
    /// use [`new`](Self::new).
    ///
    /// [scheme]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.1
    #[inline]
    #[must_use]
    pub const fn new_or_panic(s: &str) -> &Scheme {
        match Self::new(s) {
            Some(scheme) => scheme,
            None => panic!("invalid scheme"),
        }
    }

    /// Converts a string slice to `&Scheme`, returning `None` if the conversion fails.
    #[inline]
    #[must_use]
    pub const fn new(s: &str) -> Option<&Scheme> {
        if matches!(s.as_bytes(), [first, rem @ ..]
        if first.is_ascii_alphabetic() && table::SCHEME.validate(rem))
        {
            Some(Scheme::new_validated(s))
        } else {
            None
        }
    }

    /// Returns the scheme component as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::UriRef;
    ///
    /// let uri_ref = UriRef::parse("http://example.com/")?;
    /// assert_eq!(uri_ref.scheme().unwrap().as_str(), "http");
    /// let uri_ref = UriRef::parse("HTTP://EXAMPLE.COM/")?;
    /// assert_eq!(uri_ref.scheme().unwrap().as_str(), "HTTP");
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl PartialEq for Scheme {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq_ignore_ascii_case(&other.inner)
    }
}

impl Eq for Scheme {}

/// The [authority] component of URI reference.
///
/// [authority]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2
#[derive(Clone, Copy)]
pub struct Authority<'a> {
    val: &'a str,
    meta: AuthMeta,
}

impl<'a> Authority<'a> {
    #[inline]
    pub(crate) const fn new(val: &'a str, meta: AuthMeta) -> Self {
        Self { val, meta }
    }

    /// An empty authority component.
    pub const EMPTY: Authority<'static> = Authority::new("", AuthMeta::EMPTY);

    pub(crate) fn meta(&self) -> AuthMeta {
        self.meta
    }

    /// Returns the authority component as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::UriRef;
    ///
    /// let uri_ref = UriRef::parse("http://user@example.com:8080/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.as_str(), "user@example.com:8080");
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.val
    }

    /// Returns the optional [userinfo] subcomponent.
    ///
    /// [userinfo]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::{encoding::EStr, UriRef};
    ///
    /// let uri_ref = UriRef::parse("http://user@example.com/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.userinfo(), Some(EStr::new_or_panic("user")));
    ///
    /// let uri_ref = UriRef::parse("http://example.com/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.userinfo(), None);
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[must_use]
    pub fn userinfo(&self) -> Option<&'a EStr<Userinfo>> {
        let host_start = self.meta.host_bounds.0;
        (host_start != 0).then(|| EStr::new_validated(&self.val[..host_start - 1]))
    }

    /// Returns the [host] subcomponent as a string slice.
    ///
    /// The host subcomponent is always present, although it may be empty.
    ///
    /// The square brackets enclosing an IPv6 or IPvFuture address are included.
    ///
    /// Note that the host subcomponent is *case-insensitive*.
    ///
    /// [host]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::UriRef;
    ///
    /// let uri_ref = UriRef::parse("http://user@example.com:8080/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.host(), "example.com");
    ///
    /// let uri_ref = UriRef::parse("file:///path/to/file")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.host(), "");
    ///
    /// let uri_ref = UriRef::parse("//[::1]")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.host(), "[::1]");
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[must_use]
    pub fn host(&self) -> &'a str {
        let (start, end) = self.meta.host_bounds;
        &self.val[start..end]
    }

    /// Returns the parsed [host] subcomponent.
    ///
    /// Note that the host subcomponent is *case-insensitive*.
    ///
    /// [host]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::{component::Host, encoding::EStr, UriRef};
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///
    /// let uri_ref = UriRef::parse("//127.0.0.1")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert!(matches!(auth.host_parsed(), Host::Ipv4(Ipv4Addr::LOCALHOST)));
    ///
    /// let uri_ref = UriRef::parse("//[::1]")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert!(matches!(auth.host_parsed(), Host::Ipv6(Ipv6Addr::LOCALHOST)));
    ///
    /// let uri_ref = UriRef::parse("//[v1.addr]")?;
    /// let auth = uri_ref.authority().unwrap();
    /// // The API design for IPvFuture addresses is to be determined.
    /// assert!(matches!(auth.host_parsed(), Host::IpvFuture { .. }));
    ///
    /// let uri_ref = UriRef::parse("//localhost")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert!(matches!(auth.host_parsed(), Host::RegName(name) if name == "localhost"));
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[must_use]
    pub fn host_parsed(&self) -> Host<'a> {
        match self.meta.host_meta {
            #[cfg(feature = "net")]
            HostMeta::Ipv4(addr) => Host::Ipv4(addr),
            #[cfg(feature = "net")]
            HostMeta::Ipv6(addr) => Host::Ipv6(addr),

            #[cfg(not(feature = "net"))]
            HostMeta::Ipv4() => Host::Ipv4(),
            #[cfg(not(feature = "net"))]
            HostMeta::Ipv6() => Host::Ipv6(),

            HostMeta::IpvFuture => Host::IpvFuture,
            HostMeta::RegName => Host::RegName(EStr::new_validated(self.host())),
        }
    }

    /// Returns the optional [port] subcomponent.
    ///
    /// A scheme may define a default port to use when the port is
    /// not present or is empty.
    ///
    /// Note that the port may be empty, with leading zeros, or larger than [`u16::MAX`].
    /// It is up to you to decide whether to deny such ports, fallback to the scheme's
    /// default if it is empty, ignore the leading zeros, or use a special addressing
    /// mechanism that allows ports larger than [`u16::MAX`].
    ///
    /// [port]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.3
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::{encoding::EStr, UriRef};
    ///
    /// let uri_ref = UriRef::parse("//localhost:4673/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port(), Some(EStr::new_or_panic("4673")));
    ///
    /// let uri_ref = UriRef::parse("//localhost:/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port(), Some(EStr::EMPTY));
    ///
    /// let uri_ref = UriRef::parse("//localhost/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port(), None);
    ///
    /// let uri_ref = UriRef::parse("//localhost:123456/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port(), Some(EStr::new_or_panic("123456")));
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[must_use]
    pub fn port(&self) -> Option<&'a EStr<Port>> {
        let host_end = self.meta.host_bounds.1;
        (host_end != self.val.len()).then(|| EStr::new_validated(&self.val[host_end + 1..]))
    }

    /// Converts the [port] subcomponent to `u16`, if present and nonempty.
    ///
    /// Returns `Ok(None)` if the port is not present or is empty. Leading zeros are ignored.
    ///
    /// [port]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.3
    ///
    /// # Errors
    ///
    /// Returns `Err` if the port cannot be parsed into `u16`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::UriRef;
    ///
    /// let uri_ref = UriRef::parse("//localhost:4673/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port_to_u16(), Ok(Some(4673)));
    ///
    /// let uri_ref = UriRef::parse("//localhost/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port_to_u16(), Ok(None));
    ///
    /// let uri_ref = UriRef::parse("//localhost:/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert_eq!(auth.port_to_u16(), Ok(None));
    ///
    /// let uri_ref = UriRef::parse("//localhost:123456/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert!(auth.port_to_u16().is_err());
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    pub fn port_to_u16(&self) -> Result<Option<u16>, ParseIntError> {
        self.port()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str().parse())
            .transpose()
    }

    /// Converts the host and the port subcomponent to an iterator of resolved [`SocketAddr`]s.
    ///
    /// The default port is used if the port component is not present or is empty.
    /// A registered name is first [decoded] and then resolved with [`ToSocketAddrs`].
    ///
    /// [decoded]: EStr::decode
    ///
    /// # Errors
    ///
    /// Returns `Err` if any of the following is true.
    ///
    /// - The port cannot be parsed into `u16`.
    /// - The host is an IPvFuture address.
    /// - A registered name does not decode to valid UTF-8 or fails to resolve.
    #[cfg(all(feature = "net", feature = "std"))]
    pub fn socket_addrs(&self, default_port: u16) -> io::Result<impl Iterator<Item = SocketAddr>> {
        use std::vec;

        let port = self
            .port_to_u16()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid port value"))?
            .unwrap_or(default_port);

        match self.host_parsed() {
            Host::Ipv4(addr) => Ok(vec![(addr, port).into()].into_iter()),
            Host::Ipv6(addr) => Ok(vec![(addr, port).into()].into_iter()),
            Host::IpvFuture => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "address mechanism not supported",
            )),
            Host::RegName(name) => {
                let name = name.decode().into_string().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "registered name does not decode to valid UTF-8",
                    )
                })?;
                (&name[..], port).to_socket_addrs()
            }
        }
    }

    /// Checks whether the authority component contains a userinfo subcomponent.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::UriRef;
    ///
    /// let uri_ref = UriRef::parse("http://user@example.com/")?;
    /// assert!(uri_ref.authority().unwrap().has_userinfo());
    ///
    /// let uri_ref = UriRef::parse("http://example.com/")?;
    /// assert!(!uri_ref.authority().unwrap().has_userinfo());
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    #[inline]
    #[must_use]
    pub fn has_userinfo(&self) -> bool {
        self.meta.host_bounds.0 != 0
    }

    /// Checks whether the authority component contains a port subcomponent.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_uri::UriRef;
    ///
    /// let uri_ref = UriRef::parse("//localhost:4673/")?;
    /// assert!(uri_ref.authority().unwrap().has_port());
    ///
    /// // The port subcomponent can be empty.
    /// let uri_ref = UriRef::parse("//localhost:/")?;
    /// assert!(uri_ref.authority().unwrap().has_port());
    ///
    /// let uri_ref = UriRef::parse("//localhost/")?;
    /// let auth = uri_ref.authority().unwrap();
    /// assert!(!uri_ref.authority().unwrap().has_port());
    /// # Ok::<_, fluent_uri::error::ParseError>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn has_port(&self) -> bool {
        self.meta.host_bounds.1 != self.val.len()
    }
}

/// The parsed [host] component of URI reference.
///
/// [host]: https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2
#[derive(Debug, Clone, Copy)]
#[cfg_attr(fuzzing, derive(PartialEq, Eq))]
pub enum Host<'a> {
    /// An IPv4 address.
    #[cfg_attr(not(feature = "net"), non_exhaustive)]
    Ipv4(
        /// The address.
        #[cfg(feature = "net")]
        Ipv4Addr,
    ),
    /// An IPv6 address.
    #[cfg_attr(not(feature = "net"), non_exhaustive)]
    Ipv6(
        /// The address.
        #[cfg(feature = "net")]
        Ipv6Addr,
    ),
    /// An IP address of future version.
    ///
    /// This variant is marked as non-exhaustive because the API design
    /// for IPvFuture addresses is to be determined.
    #[non_exhaustive]
    IpvFuture,
    /// A registered name.
    ///
    /// Note that registered names are *case-insensitive*.
    RegName(&'a EStr<RegName>),
}
