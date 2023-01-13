//! Small representations for sets of (mostly) known strings using enums.
use std::{
    borrow::Borrow, cmp::Ordering, convert::Infallible, fmt, hash, ops::Deref, str::FromStr,
};

pub use smol_str::SmolStr;

pub use smoller_str_macro::*;

// #[macro_export]
// macro_rules! include_smoller_strings {
//     ($name:ident, $file:literal) => {
//         $crate::include_smoller_strings_inner!($name, concat!(file!(), "/", $file))
//     };
// }

/// An enum representing a set of statically known strings.
pub trait EnumStr: FromStr + Copy + fmt::Display + 'static {
    const VALUES: &'static [Self];
    fn as_str(&self) -> &'static str;
    // fn values(&self) -> &'static [Self];
}

pub trait SmollerStr: FromStr<Err = Infallible> + Clone + fmt::Display + 'static {
    const BUILTIN: &'static [Self];

    fn new<S: AsRef<str> + ?Sized>(value: &S) -> Self;
    fn as_str(&self) -> &str;
    fn is_heap_allocated(&self) -> bool;
    fn is_builtin_value(&self) -> bool;
}

pub enum Repr<E: EnumStr> {
    Smol(SmolStr),
    Smoller(E),
}

impl<E: EnumStr> hash::Hash for Repr<E> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher)
    }
}

impl<E: EnumStr> Deref for Repr<E> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<E: EnumStr> Borrow<str> for Repr<E> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

macro_rules! partial_eq {
    ($t:ty, $self_fn:ident $(, $other_fn:ident)?) => {
        impl<E: EnumStr> PartialEq<$t> for Repr<E> {
            fn eq(&self, other: &$t) -> bool {
                self.$self_fn() == other $(.$other_fn())?
            }
        }
        impl<E: EnumStr> PartialEq<Repr<E>> for $t {
            fn eq(&self, other: &Repr<E>) -> bool {
                self $(.$other_fn())? == other.$self_fn()
            }
        }
        impl<'a, E: EnumStr> PartialEq<&'a $t> for Repr<E> {
            fn eq(&self, other: &&'a $t) -> bool {
                self == *other
            }
        }
        impl<'a, E: EnumStr> PartialEq<Repr<E>> for &'a $t {
            fn eq(&self, other: &Repr<E>) -> bool {
                *self == other
            }
        }
    };
}

impl<E: EnumStr> PartialEq for Repr<E> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

partial_eq! {str, as_str}
partial_eq! {String, as_str, as_str}
partial_eq! {SmolStr, as_str, as_str}

impl<E: EnumStr> Eq for Repr<E> {}

impl<E: EnumStr> Ord for Repr<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<E: EnumStr> PartialOrd for Repr<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: EnumStr> Repr<E> {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Smol(s) => s.as_str(),
            Self::Smoller(s) => s.as_str(),
        }
    }
}

impl<E: EnumStr> fmt::Debug for Repr<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<E: EnumStr> fmt::Display for Repr<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}
