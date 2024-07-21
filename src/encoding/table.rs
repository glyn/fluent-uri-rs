//! Byte pattern tables from RFC 3986.
//!
//! The predefined table constants in this module are documented with
//! the ABNF notation of [RFC 5234].
//!
//! [RFC 5234]: https://datatracker.ietf.org/doc/html/rfc5234

use super::Table;
use alloc::string::String;

const fn gen_hex_table() -> [u8; 512] {
    const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

    let mut i = 0;
    let mut out = [0; 512];
    while i < 256 {
        out[i * 2] = HEX_DIGITS[i >> 4];
        out[i * 2 + 1] = HEX_DIGITS[i & 0b1111];
        i += 1;
    }
    out
}

const HEX_TABLE: &[u8; 512] = &gen_hex_table();

impl Table {
    /// Generates a table that only allows the given unencoded bytes.
    ///
    /// # Panics
    ///
    /// Panics if any of the bytes is not ASCII or equals `b'%'`.
    #[must_use]
    pub const fn gen(mut bytes: &[u8]) -> Table {
        let mut arr = [0; 256];
        while let [cur, rem @ ..] = bytes {
            assert!(
                cur.is_ascii() && *cur != b'%',
                "cannot allow non-ASCII byte or %"
            );
            arr[*cur as usize] = 1;
            bytes = rem;
        }
        Table {
            arr,
            allows_enc: false,
        }
    }

    /// Marks this table as allowing percent-encoded octets.
    #[must_use]
    pub const fn enc(mut self) -> Table {
        self.allows_enc = true;
        self
    }

    /// Combines two tables into one.
    ///
    /// Returns a new table that allows all the byte patterns allowed
    /// by `self` or by `other`.
    #[must_use]
    pub const fn or(mut self, other: &Table) -> Table {
        let mut i = 0;
        while i < 256 {
            self.arr[i] |= other.arr[i];
            i += 1;
        }
        self.allows_enc |= other.allows_enc;
        self
    }

    /// Subtracts from this table.
    ///
    /// Returns a new table that allows all the byte patterns allowed
    /// by `self` but not allowed by `other`.
    #[must_use]
    pub const fn sub(mut self, other: &Table) -> Table {
        let mut i = 0;
        while i < 256 {
            if other.arr[i] != 0 {
                self.arr[i] = 0;
            }
            i += 1;
        }
        if other.allows_enc {
            self.allows_enc = false;
        }
        self
    }

    /// Checks whether the table is a subset of another, i.e., `other`
    /// allows at least all the byte patterns allowed by `self`.
    #[must_use]
    pub const fn is_subset(&self, other: &Table) -> bool {
        let mut i = 0;
        while i < 256 {
            if self.arr[i] != 0 && other.arr[i] == 0 {
                return false;
            }
            i += 1;
        }
        !self.allows_enc || other.allows_enc
    }

    /// Returns the specified table value.
    #[inline]
    pub(crate) const fn get(&self, x: u8) -> u8 {
        self.arr[x as usize]
    }

    /// Checks whether the given unencoded byte is allowed by the table.
    #[inline]
    #[must_use]
    pub const fn allows(&self, x: u8) -> bool {
        self.get(x) != 0
    }

    /// Checks whether percent-encoded octets are allowed by the table.
    #[inline]
    #[must_use]
    pub const fn allows_enc(&self) -> bool {
        self.allows_enc
    }

    #[inline]
    pub(crate) fn encode(&self, x: u8, buf: &mut String) {
        if self.allows(x) {
            buf.push(x as char);
        } else {
            buf.push('%');
            buf.push(HEX_TABLE[x as usize * 2] as char);
            buf.push(HEX_TABLE[x as usize * 2 + 1] as char);
        }
    }

    /// Validates the given byte sequence with the table.
    pub(crate) const fn validate(&self, s: &[u8]) -> bool {
        let mut i = 0;
        if self.allows_enc() {
            while i < s.len() {
                let x = s[i];
                if x == b'%' {
                    if i + 2 >= s.len() {
                        return false;
                    }
                    let (hi, lo) = (s[i + 1], s[i + 2]);

                    if HEXDIG.get(hi) & HEXDIG.get(lo) == 0 {
                        return false;
                    }
                    i += 3;
                } else {
                    if !self.allows(x) {
                        return false;
                    }
                    i += 1;
                }
            }
        } else {
            while i < s.len() {
                if !self.allows(s[i]) {
                    return false;
                }
                i += 1;
            }
        }
        true
    }
}

const fn gen(bytes: &[u8]) -> Table {
    Table::gen(bytes)
}

/// `ALPHA = %x41-5A / %x61-7A`
pub const ALPHA: &Table = &gen(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");

/// `DIGIT = %x30-39`
pub const DIGIT: &Table = &gen(b"0123456789");

/// `HEXDIG = DIGIT / "A" / "B" / "C" / "D" / "E" / "F"`
pub const HEXDIG: &Table = &DIGIT.or(&gen(b"ABCDEFabcdef"));

/// `scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )`
pub const SCHEME: &Table = &ALPHA.or(DIGIT).or(&gen(b"+-."));

/// `userinfo = *( unreserved / pct-encoded / sub-delims / ":" )`
pub const USERINFO: &Table = &UNRESERVED.or(SUB_DELIMS).or(&gen(b":")).enc();

/// `IPvFuture = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )`
pub const IPV_FUTURE: &Table = &UNRESERVED.or(SUB_DELIMS).or(&gen(b":"));

/// `reg-name = *( unreserved / pct-encoded / sub-delims )`
pub const REG_NAME: &Table = &UNRESERVED.or(SUB_DELIMS).enc();

/// `path = *( pchar / "/" )`
pub const PATH: &Table = &PCHAR.or(&gen(b"/"));

/// `segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )`
pub const SEGMENT_NZ_NC: &Table = &UNRESERVED.or(SUB_DELIMS).or(&gen(b"@")).enc();

/// `pchar = unreserved / pct-encoded / sub-delims / ":" / "@"`
pub const PCHAR: &Table = &UNRESERVED.or(SUB_DELIMS).or(&gen(b":@")).enc();

/// `query = *( pchar / "/" / "?" )`
pub const QUERY: &Table = &PCHAR.or(&gen(b"/?"));

/// `fragment = *( pchar / "/" / "?" )`
pub const FRAGMENT: &Table = QUERY;

/// `unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"`
pub const UNRESERVED: &Table = &ALPHA.or(DIGIT).or(&gen(b"-._~"));

/// `reserved = gen-delims / sub-delims`
pub const RESERVED: &Table = &GEN_DELIMS.or(SUB_DELIMS);

/// `gen-delims = ":" / "/" / "?" / "#" / "[" / "]" / "@"`
pub const GEN_DELIMS: &Table = &gen(b":/?#[]@");

/// `sub-delims = "!" / "$" / "&" / "'" / "(" / ")"
///             / "*" / "+" / "," / ";" / "="`
pub const SUB_DELIMS: &Table = &gen(b"!$&'()*+,;=");
