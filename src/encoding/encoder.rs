#![allow(missing_debug_implementations)]

//! Percent-encoders for URI components.

use super::{table::*, Encoder, Table};

/// An encoder for userinfo.
pub struct Userinfo(());

impl Encoder for Userinfo {
    const TABLE: &'static Table = USERINFO;
}

/// An encoder for registered name.
pub struct RegName(());

impl Encoder for RegName {
    const TABLE: &'static Table = REG_NAME;
}

/// An encoder for port.
pub struct Port(());

impl Encoder for Port {
    const TABLE: &'static Table = DIGIT;
}

/// An encoder for path.
///
/// [`EStr`] has [extension methods] for the path component.
///
/// [`EStr`]: super::EStr
/// [extension methods]: super::EStr#impl-EStr<Path>
pub struct Path(());

impl Encoder for Path {
    const TABLE: &'static Table = PATH;
}

/// An encoder for query.
pub struct Query(());

impl Encoder for Query {
    const TABLE: &'static Table = QUERY;
}

/// An encoder for fragment.
pub struct Fragment(());

impl Encoder for Fragment {
    const TABLE: &'static Table = FRAGMENT;
}

/// An encoder for data which preserves only [unreserved] characters
/// and encodes the others.
///
/// [unreserved]: https://datatracker.ietf.org/doc/html/rfc3986#section-2.3
pub struct Data(());

impl Encoder for Data {
    const TABLE: &'static Table = &UNRESERVED.enc();
}
