// Bitcoin secp256k1 bindings
// Written in 2014 by
//   Dawid Ciężarkiewicz
//   Andrew Poelstra
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

// This is a macro that routinely comes in handy
macro_rules! impl_array_newtype {
    ($thing:ident, $ty:ty, $len:expr) => {
        impl $thing {
            #[inline]
            /// Converts the object to a raw pointer for FFI interfacing
            pub fn as_ptr(&self) -> *const $ty {
                let &$thing(ref dat) = self;
                dat.as_ptr()
            }

            #[inline]
            /// Converts the object to a mutable raw pointer for FFI interfacing
            pub fn as_mut_ptr(&mut self) -> *mut $ty {
                let &mut $thing(ref mut dat) = self;
                dat.as_mut_ptr()
            }

            #[inline]
            /// Returns the length of the object as an array
            pub fn len(&self) -> usize { $len }

            #[inline]
            /// Returns whether the object as an array is empty
            pub fn is_empty(&self) -> bool { false }
        }

        impl AsRef<[u8]> for $thing {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl PartialEq for $thing {
            #[inline]
            fn eq(&self, other: &$thing) -> bool {
                &self[..] == &other[..]
            }
        }

        impl Eq for $thing {}

        impl PartialOrd for $thing {
            #[inline]
            fn partial_cmp(&self, other: &$thing) -> Option<::core::cmp::Ordering> {
                self[..].partial_cmp(&other[..])
            }
        }

        impl Ord for $thing {
            #[inline]
            fn cmp(&self, other: &$thing) -> ::core::cmp::Ordering {
                self[..].cmp(&other[..])
            }
        }

        impl Clone for $thing {
            #[inline]
            fn clone(&self) -> $thing {
                unsafe {
                    use std::ptr::copy_nonoverlapping;
                    use std::mem;
                    let mut ret: $thing = mem::MaybeUninit::uninit().assume_init();
                    copy_nonoverlapping(self.as_ptr(),
                                        ret.as_mut_ptr(),
                                        mem::size_of::<$thing>());
                    ret
                }
            }
        }

        impl ::std::ops::Index<usize> for $thing {
            type Output = $ty;

            #[inline]
            fn index(&self, index: usize) -> &$ty {
                let &$thing(ref dat) = self;
                &dat[index]
            }
        }

        impl ::std::ops::Index<::std::ops::Range<usize>> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, index: ::std::ops::Range<usize>) -> &[$ty] {
                let &$thing(ref dat) = self;
                &dat[index]
            }
        }

        impl ::std::ops::Index<::std::ops::RangeTo<usize>> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, index: ::std::ops::RangeTo<usize>) -> &[$ty] {
                let &$thing(ref dat) = self;
                &dat[index]
            }
        }

        impl ::std::ops::Index<::std::ops::RangeFrom<usize>> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, index: ::std::ops::RangeFrom<usize>) -> &[$ty] {
                let &$thing(ref dat) = self;
                &dat[index]
            }
        }

        impl ::std::ops::Index<::std::ops::RangeFull> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, _: ::std::ops::RangeFull) -> &[$ty] {
                let &$thing(ref dat) = self;
                &dat[..]
            }
        }

        impl ::std::hash::Hash for $thing {
          fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
            state.write(&self.0)
            // for n in 0..self.len() {
            //   state.write_u8(self.0[n]);
            // }
          }
        }

        #[cfg(feature = "rustc-serialize")]
        impl crate::serialize::Decodable for $thing {
            fn decode<D: crate::serialize::Decoder>(d: &mut D) -> Result<$thing, D::Error> {
                use crate::serialize::Decodable;

                d.read_seq(|d, len| {
                    if len != $len {
                        Err(d.error("Invalid length"))
                    } else {
                        unsafe {
                            use std::mem;
                            let mut ret: [$ty; $len] = mem::MaybeUninit::uninit().assume_init();
                            for i in 0..len {
                                ret[i] = d.read_seq_elt(i, |d| Decodable::decode(d))?;
                            }
                            Ok($thing(ret))
                        }
                    }
                })
            }
        }

        #[cfg(feature = "rustc-serialize")]
        impl crate::serialize::Encodable for $thing {
            fn encode<S: crate::serialize::Encoder>(&self, s: &mut S)
                                               -> Result<(), S::Error> {
                self[..].encode(s)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for $thing {
            fn deserialize<D>(d: D) -> Result<$thing, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                // We have to define the Visitor struct inside the function
                // to make it local ... all we really need is that it's
                // local to the macro, but this works too :)
                struct Visitor {
                    marker: ::std::marker::PhantomData<$thing>,
                }
                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $thing;

                    #[inline]
                    fn visit_seq<A>(self, mut a: A) -> Result<$thing, A::Error>
                        where A: ::serde::de::SeqAccess<'de>
                    {
                        unsafe {
                            use std::mem;
                            let mut ret: [$ty; $len] = mem::MaybeUninit::uninit().assume_init();
                            for i in 0..$len {
                                ret[i] = match a.next_element()? {
                                    Some(c) => c,
                                    None => return Err(::serde::de::Error::invalid_length(i, &self))
                                };
                            }
                            let one_after_last : Option<u8> = a.next_element()?;
                            if one_after_last.is_some() {
                                return Err(::serde::de::Error::invalid_length($len + 1, &self));
                            }
                            Ok($thing(ret))
                        }
                    }

                    fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        write!(f, "a sequence of {} elements", $len)
                    }
                }

                // Begin actual function
                d.deserialize_seq(Visitor { marker: ::std::marker::PhantomData })
            }
        }

        #[cfg(feature = "serde")]
        impl ::serde::Serialize for $thing {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                (&self.0[..]).serialize(s)
            }
        }
    }
}

macro_rules! impl_pretty_debug {
    ($thing:ident) => {
        impl ::std::fmt::Debug for $thing {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}(", stringify!($thing))?;
                for i in self[..].iter().cloned() {
                    write!(f, "{:02x}", i)?;
                }
                write!(f, ")")
            }
        }
     }
}

macro_rules! impl_raw_debug {
    ($thing:ident) => {
        impl ::std::fmt::Debug for $thing {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                for i in self[..].iter().cloned() {
                    write!(f, "{:02x}", i)?;
                }
                Ok(())
            }
        }
     }
}

macro_rules! map_vec {
  ($thing:expr, $mapfn:expr ) => {
    $thing.iter()
      .map($mapfn)
      .collect::<Vec<_>>();
  }
}

#[cfg(test)]
// A macro useful for serde (de)serialization tests
macro_rules! round_trip_serde (
    ($var:ident) => ({
        let start = $var;
        let mut encoded = Vec::new();
        {
            let mut serializer = crate::json::ser::Serializer::new(&mut encoded);
            ::serde::Serialize::serialize(&start, &mut serializer).unwrap();
        }
        let mut deserializer = crate::json::de::Deserializer::from_slice(&encoded);
        let decoded = ::serde::Deserialize::deserialize(&mut deserializer);
        assert_eq!(Some(start), decoded.ok());
    })
);
