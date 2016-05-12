- Feature Name: Refactor `ImmutableData::name()` Function
- Status: proposed
- Type: enhancement
- Related components: Routing, SAFE Vault and SAFE Core
- Start Date: 11-05-2016
- RFC PR:
- Issue number:

# Summary

This RFC proposes some alternative implementations for `ImmutableData::name()`.

# Motivation

The existing `ImmutableData::name()` function is highly inefficient as it performs a SHA512 hash
operation on every call.  It has to do this since it doesn't have a `name` member variable.  This
came about to satisfy the previous requirement that `ImmutableData` derived the `RustcEncodable` and
`RustcDecodable` serialisation traits rather than implementing them by hand, along with the
requirement that the serialised data contained just the value and not the name.

This RFC proposes some alternatives which are more efficient, and in one case offers improved
safety.  They also have the advantage that `name()` will return `&XorName` or `Ref<XorName>` which
is more canonical than the current return of `XorName`.

The alternatives all have a member variable `name` which increases the size of the data in memory by
64 bytes (probably insignificant in comparison to the `value` member).  However, this member
variable is not serialised in any of the implementations in order to maintain the existing level of
space-efficiency.  This requires either manual implementation of the serialisation traits or custom
encoding functions.

The proposed alternatives are:

* [Lazy][1]: lazy initialisation of an optional `name` member variable
* [Minimal][2]: non-optional `name` member variable which gets set whenever `ImmutableData` is
  constructed
* [Safe][3]: similar to Minimal, but also validates against expected name when deserialising into
  an instance of `ImmutableData` guaranteeing that `ImmutableData` instances will always be valid,
  uncorrupted chunks

## Comparison

| Type          | Can be Invalid | Contents Hashed        | Needs Custom Encode Functions <sup>1</sup> | Can Derive `Ord`, `PartialOrd` and `Hash` |
|:--------------|:---------------|:-----------------------|:-------------------------------------------|:------------------------------------------|
| [Existing][0] | Yes            | every call to `name()` | No                                         | Yes                                       |
| [Lazy][1]     | Yes            | maximum once           | No                                         | No                                        |
| [Minimal][2]  | Yes            | exactly once           | No                                         | Yes                                       |
| [Safe][3]     | No             | exactly once           | Yes                                        | Yes                                       |

1: We can't use the `serialise()` and `deserialise()` functions from maidsafe_utilities for any that
need custom encode functions, as they won't implement the required traits.

# Detailed design

There is an example implementation along with tests and benchmark code at
https://gitlab.com/Fraser999/DataName.

## <a name="Existing"></a>Existing Implementation

For comparison, the existing (abbreviated) implementation of `ImmutableData` is:

```rust
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd, RustcEncodable, RustcDecodable)]
pub struct ImmutableData {
    value: Vec<u8>,
}

impl ImmutableData {
    pub fn new(value: Vec<u8>) -> ImmutableData {
        ImmutableData { value: value }
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn name(&self) -> XorName {
        XorName(sha512::hash(&self.value).0)
    }
}
```

---

## <a name="Lazy"></a>Lazy Implementation

```rust
#[derive(Clone, Eq, PartialEq)]
pub struct ImmutableData {
    name: RefCell<XorName>,
    name_initialised: Cell<bool>,
    value: Vec<u8>,
}

impl ImmutableData {
    pub fn new(value: Vec<u8>) -> ImmutableData {
        ImmutableData {
            name: RefCell::new(XorName::new([0; XOR_NAME_LEN])),
            name_initialised: Cell::new(false),
            value: value,
        }
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn name(&self) -> Ref<XorName> {
        if !self.name_initialised.get() {
            *self.name.borrow_mut() = XorName(sha512::hash(&self.value).0);
            self.name_initialised.set(true);
        }
        self.name.borrow()
    }

    pub fn validate(&mut self, expected_name: &XorName) -> bool {
        *self.name() == *expected_name
    }
}

impl Encodable for ImmutableData {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.value.encode(encoder)
    }
}

impl Decodable for ImmutableData {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<ImmutableData, D::Error> {
        let value: Vec<u8> = try!(Decodable::decode(decoder));
        Ok(ImmutableData {
            name: RefCell::new(XorName::new([0; XOR_NAME_LEN])),
            name_initialised: Cell::new(false),
            value: value,
        })
    }
}
```

---

## <a name="Minimal"></a>Minimal Implementation

```rust
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImmutableData {
    name: XorName,
    value: Vec<u8>,
}

impl ImmutableData {
    pub fn new(value: Vec<u8>) -> ImmutableData {
        ImmutableData {
            name: XorName(sha512::hash(&value).0),
            value: value,
        }
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn name(&self) -> &XorName {
        &self.name
    }

    pub fn validate(&self, expected_name: &XorName) -> bool {
        self.name == *expected_name
    }
}

impl Encodable for ImmutableData {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
        self.value.encode(encoder)
    }
}

impl Decodable for ImmutableData {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<ImmutableData, D::Error> {
        let value: Vec<u8> = try!(Decodable::decode(decoder));
        let name = XorName(sha512::hash(&value).0);
        Ok(ImmutableData {
            name: name,
            value: value,
        })
    }
}
```

---

## <a name="Safe"></a>Safe Implementation

```rust
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ImmutableData {
    name: XorName,
    value: Vec<u8>,
}

impl ImmutableData {
    pub fn new(value: Vec<u8>) -> ImmutableData {
        ImmutableData {
            name: XorName(sha512::hash(&value).0),
            value: value,
        }
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    pub fn name(&self) -> &XorName {
        &self.name
    }

    pub fn serialise(&self) -> Result<Vec<u8>, DataError> {
        Ok(try!(serialisation::serialise(&self.value)))
    }

    pub fn deserialise(serialised_safe: &[u8], expected_name: &XorName) -> Result<ImmutableData, DataError> {
        let value = try!(serialisation::deserialise::<Vec<u8>>(serialised_safe));
        let name = XorName(sha512::hash(&value).0);
        if name == *expected_name {
            Ok(ImmutableData {
                name: name,
                value: value,
            })
        } else {
            Err(DataError::Validation)
        }
    }
}
```

This uses a new error type which could be implemented like:

```rust
#[derive(Debug)]
pub enum DataError {
    /// Failure to serialise or deserialise
    Encoding(SerialisationError),
    /// Invalid contents
    Validation,
}

impl From<SerialisationError> for DataError {
    fn from(error: SerialisationError) -> DataError {
        DataError::Encoding(error)
    }
}
```

---

## Sample Benchmarks for 1MB Chunks

These results are from a 64-bit Windows 10 machine with an Intel Core i7-4790K and 8GB RAM.

| Type          | Benchmark for `name()`         | Benchmark for Serialisation     | Benchmark for Deserialisation   |
|:--------------|-------------------------------:|--------------------------------:|--------------------------------:|
| [Existing][0] | 2,487,253 ns/iter (+/- 64,476) | 2,313,759 ns/iter (+/- 41,182)  | 5,479,790 ns/iter (+/- 270,810) |
| [Lazy][1]     |   124,710 ns/iter (+/- 3,648)  | 2,339,700 ns/iter (+/- 159,777) | 5,483,177 ns/iter (+/- 183,479) |
| [Minimal][2]  |         0 ns/iter (+/- 1)      | 2,314,490 ns/iter (+/- 29,752)  | 8,081,331 ns/iter (+/- 183,989) |
| [Safe][3]     |         0 ns/iter (+/- 0)      | 3,026,149 ns/iter (+/- 89,004)  | 7,237,680 ns/iter (+/- 108,792) |

# Drawbacks

None over existing implementation.

# Alternatives

None.

# Unresolved questions

Which of the alternatives should be used?

[0]: #Existing "Source for existing `ImmutableData` implementation."
[1]: #Lazy "Source for "Lazy" implementation."
[2]: #Minimal "Source for "Minimal" implementation."
[3]: #Safe "Source for "Safe" implementation."
