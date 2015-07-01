- Feature Name: impact_unified_structured_data_on_routing
- Type: Enhancement
- Related components: routing and sentinel
- Start Date: 30-06-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

[RFC Unified Structured Data](https://github.com/dirvine/rfcs/blob/unified-structured-data/proposed/0000-Unified-structured-data.md) introduces `StructuredData` as a fundamental type for the network.
This RFC explores in more detail the implications for applying this change for routing
and sentinel library.

# Motivation

For the motivation on introducing `Unified Structured Data` we refer back to
the parent RFC.  The motivation for the current RFC is to establish a collective
understanding of the changes needed for routing and sentinel to
implementing the parent RFC.  This RFC explicitly excludes the intend to change
the design of the actual `StructuredData` and any such discussions should be
posted on the parent RFC.

# Detailed design

For the design of the actual `StructuredData` we again refer to the parent RFC.

## Types
In routing three new fundamental types will be introduced `PlainData`,
`ImmutableData`, `StructuredData`

### Plain data

```rust
struct PlainData {
    key : NameType,
    value : Vec<u8>
}
```

Routing will not perform any additional validation on the PlainData type.
Default routing behaviour applies to this type, which is briefly recapitulated
below.

### Immutable data

``` rust
struct ImmutableData {
    tag_type : u8,
    value : Vec<u8>
}

impl ImmutableData {
    pub fn new(tag_type: u8, value : Vec<u8>) -> ImmutableData {}

    pub fn name(&self) -> NameType {
        (tag_type + 1) Hash iterations of self.value
    }

    // add lifetime if needed
    pub fn content(&self) -> &[u8] {
        &self.value[..]
    }
}
```

Routing considers `ImmutableData` valid when a requested `name` equals
`immutable_data.name()`.

### Structured data

We repeat the structure as defined in the parent RFC

``` rust
struct StructuredData {
   tag_type : [u8; 8],
   identifier : NameType,
   data : Vec<u8>,
   owner_keys : Vec<crypto::sign::PublicKey>,
   version : u64,
   previous_owner_keys : vec<crypto::sign::PublicKey>
   signature : Vec<Signature>
}

impl StructuredData {
    pub fn new(
        tag_type : [u8; 8],
        identifier : NameType,
        data : Vec<u8>,
        owner_keys : Vec<crypto::sign::PublicKey>,
        version : u64,
        previous_owner_keys : vec<crypto::sign::PublicKey>
        signature : Vec<Signature>) -> Result<StructuredData> {
        // validate:
        // 0. total size <= 100 kB
        // 1. must always be owned
        // 2. on version == 0, no signatures needed
        // 3. if previous owners set: check signatures with majority
        //    of their keys
        // 4. if no previous owners: check signatures with majority
        //    of their keys
        construct!()
    }

    pub fn name(&self) -> NameType {
        SHA512(tag_type + identifier)
    }

    pub fn content(&self) -> &[u8] {}

    pub fn add_signature(&mut self, private_sign_key : &crypto::sign::SecretKey) {}
}
```

- `data` Routing does not parse the `data` field; it is always considered
as serialised bytes.
- `version` Routing does not (currently) attempt to resolve concurreny issues;
as such `version` is unused at routing.  Additionally routing has no knowledge
of the previously stored valid version. The version number is for the user.
- `tag_type` Routing does not attach meaning to the 8 byte `tag_type` and treats
all tag_types equal.
- the signature should sign in order `data`, `version`, `owner_keys` and
`previous_owner_keys`

## Impact on routing message

Currently `RoutingMessage` has the following declaration with
an obligatory signature from the `Header::Source::fromNode`
```rust
pub struct RoutingMessage {
    pub message_type: MessageTypeTag,
    pub message_header: message_header::MessageHeader,
    pub serialised_body: Vec<u8>,
    pub signature : types::Signature
}
```

As every client connects to the network over a relocated relay node,
we can keep this obligatory signature and have it signed by the relay node.
This puts responsibility on the relay node when injecting client messages in
the network, and keeps consistency that the from_node on the network is always
the id of a relocated node, not the hash of a 32byte client PublicKey.

RoutingClient will sign RoutingMessages with the generic unrelocated Id
it self-generates on construction.  This generic Id is unrelated to the keys
for signing ownership of structured data; it is purely and internally used
by routing to identify client-relay connections, per session.

##

### Standard routing behaviour

    <A|B|C>

    MessageAction::SendOn

# Drawbacks

Why should we *not* do this?

# Alternatives

What other designs have been considered? What is the impact of not doing this?

# Unresolved questions

What parts of the design are still to be done?
