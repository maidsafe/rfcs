- Feature Name: Unify Structured  data
- Type: Enhancement
- Related components: routing, maidsafe_types, maidsafe_vault, maidsafe_client, sentinel
- Start Date: 13-06-2015
- Issue number: #27

# Summary

Have network only recognise two primary data types, Immutable and Structured. These types will have tag_ids to allow them to contain several data types that can be used in the network by users of the client interface.
This does mean a change to default behaviour and is, therefore a significant change. ImmutableData has already two sub-types (Backup and Sacrificial). This proposal should simplify the sentinel and interfaces from routing to users of routing as there will be no need to pass down type information (i.e. how to get the name or owner etc.). These types can actually be defined in the routing library, allowing users of the library to use the `type_tag` to create their own types and actions on those types.

# Motivation

## Why?

The primary goal is two fold, reduce network traffic (by removing an indirection, of looking up a value
and using that as a key to lookup next) and also to remove complexity (thereby increasing security).

Another facet of this proposal is extendibility. In networks such as SAFE for instance, client app developers can define their own types (say of the `fix` protocol for financial transactions) and instantiate this type on the network. For users creating their own network they may white list or blacklist types and type_id's as they wish, but the possibility would exist for network builders (of new networks) to allow extensibility of types.  

## What cases does it support?

This change supports all use of non immutable data (structured data). This covers all non `content only` data
on the network and how it is handled.

### Data storage and retrieval

ImmutableData is fixed self validating non mutable chunks. These require StructuredData types to manipulate information. These structured Data types may then create a global application acting on a key value store with very high degrees of availability and security (i.e. create network scale apps). Such apps could easily include medical condition analysis linked with genomic and proteomic sequencing to advance health based knowledge on a global scale. This proposal allows such systems to certainly be prototyped and tested with a high degree of flexibility.

### New protocols

As these data types are now self validating and may contain different information, such as new protocols, `rdf`/`owl` data types, the limit of new data types and ability to link such data is extremely scalable. Such protocols could indeed easily encompass token based systems (a form of 'crypto-currency'), linked data, natural language learning databases, pre-compilation units, distributed version control systems (git like) etc.

### Compute

Such a scheme would allow global computation types, possibly a Domain Specific Language (DSL) would define operator types to allow combination of functions. These could be made monotonic and allow out of order processing of programs (disorderly programming) which in itself presents an area that may prove to be well aligned with decentralised 'intelligence' efforts. Linked with 'zk-snarks' to alleviate any 'halting problem' type issues then a global Turing complete programming environment that optionally acts on semantic ('owl' / 'json-ld' etc.) data is a possible.

## Expected outcome


It is expected removing Transaction Managers from network will reduce complexity, code and increase security on the network, whilst allowing a greater degree of flexibility. These types now allow the users of this network to create their own data types and structures to be securely managed. This is hoped to allow many new types of application to exist.

# Detailed design

The design entails reducing all StructuredData types to a single type, therefore it should be able to be recognised by the network as StructuredData and all such sub-types handled exactly in the same manner.

## StructuredData

```
struct StructuredData {
tag_type : TagType, // 8 Bytes ?
identifier : NameType // 64Bytes (i.e. SHA512 Hash)
data : mut Vec<u8>, // in many cases this is encrypted
owner_keys : mut vec<crypto::sign::PublicKey> // n * 32 Bytes (where n is number of owners)
version : mut u64, // incrementing (deterministic) version number
previous_owner_keys : mut vec<crypto::sign::PublicKey> // n * 32 Bytes (where n is number of
owners) only required when owners change
signature : mut Vec<Signature> // signs the `mut` fields above // 32 bytes (using e25519 sig)
}
```

Fixed (immutable fields)
- tag_type
- identifier

## Validation

- To confirm name (storage location on network) we SHA512(tag_type + identifier). As these are much
  smaller than the hash it prevents flooding of a network location.
- To validate data we confirm signature using hash of (tag_type + version) as nonce. Initial `Put`
  does not require this signature, but does require the owner contain a value.
- If previous owners is set then the signature is confirmed using those keys (allows owner
  change/transfer etc.). In this case the previous owners is only required on first update and may be
  remove in next version if the owners are not changed again.
- To confirm sender of any `Put` (store or overwrite) then we check the signature of sender using same mechanism. For multiple senders we confirm at least 50% of owners have signed the request for `Put`

When `Put` on the network this type is `StructuredData` with a subtype field. The network ignores this subtype
except for collisions. No two data types with the same name and type can exist on the network.

These types are stored in a disk based storage mechanism such as `Btree` at the NaeManagers (DataManagers) responsible for the area of the network at that `name`.

These types will be limited to 100kB in size (as Immutable Chunks are also limited to 1Mb) which is for the time being a magic number.

If a client requires these be larger than 100kB then the data component will contain a (optionally encrypted) datamap to be able to retrieve chunks of the network.

The network will accept these types if `Put` by a Group and contains a message signed by at least 50% of owners as indicated. For avoidance of doubt 2 owners would require at least 1 have signed, 4 owners would require at least 2 etc. for majority control use an odd number of owners. Any `Put` must obey the mutability rules of these types.

To update such a type the client will `Post` direct (not paying for this again) and the network will overwrite the existing data element if the request is signed by the owner and the version increments. To update a type then there must be an existing type of the same `Identity` and `type` whose owners (or optionally previous owners) includes at least a majority of this new type.

For private data the data filed will be encrypted (at client discretion), for public data this need not be the case as anyone can read that, but only the owner can update it.

## Client perspective

- Decide on a `type_tag` for a new type.
- use whichever mechanism to create an `Identity` for this type
- serialise any structure into `Vec<u8>` and include in data field (can be any structure that is serialisable)
- store on network via `routing::Put(Identity: location, Data::StructuredData : data, u64: type_tag);`
- Get from network via `routing::Get(Identity: name, Data::type : type, u64: type_tag);`
- Mutate on network via `routing::Put(Identity: location, Data::StructuredData : data, u64: type_tag);`
- Delete from network via `routing::Delete(Identity: name, Data::type : type, u64: type_tag);`
## Security

### Replay attack avoidance

The inclusion of the version number will provide resistance to replay attacks.

### GetKey removal

The removal and validation of client keys is also a significant reduction in complexity and means instead of lookups to get keys, these keys are included as part of the data. This makes the data self validating and reduces security risks from Spartacus type attacks. It also removes ability for any key replacement attack on data.

# Drawbacks

This will put a heavier requirement on `Refresh` calls on the network in times of churn, rather than transferring only keys and versions (which may be small) this will require sending up to 100kB per StructuredData element. If content was always held as immutable data then it is not transferred at every churn event.
The client will also have more work as when the StructuredData type is larger than 100kB it then has to self_encrypt the remainder and store the datamap in the data filed of the StructuredData. This currently happens in a manner, but every time and without calculation of when.

# Alternatives

Status quo is an option and realistic.

- Another option posed by Qi is that structured data types be directly stored on `ManagedNodes` (`PmidNode`), this would reduce churn issues, but may introduce a security concern as we do not trust a `Pmidnode` to not lie or replay and it potentially can with data that is mutable and not intrinsically validatable as accurate.
- Another possibility by Qi is the `PmidManager` (`NodeManager`) stores the data size separate from the immutableData size.  


# Unresolved questions

1. Size of StructuredData packet, it would be nice if it were perhaps 512Bytes to have the best chance to fit into a single UDP packet, although not guaranteed. Means a payload after serialisation of only a few hundred bytes (maybe less)
2. Version conflicts or out of order updates, will upper layers handle this via a wait condition?

--------------------------------------------------------------------------------
# Addendum

- Feature Name: impact_unified_structured_data_on_routing
- Type: Enhancement
- Related components: routing and sentinel
- Start Date: 30-06-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

[RFC Unified Structured Data](https://github.com/dirvine/rfcs/blob/unified-structured-data/proposed/0000-Unified-structured-data.md) introduces `StructuredData` as a fundamental type for the network.
This RFC explores in more detail the implications for applying this change for routing
and sentinel library.  This is not the exact implementation as completed after Rust-3, but appended for reference.

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
   tag_type : u64,
   identifier : NameType,
   data : Vec<u8>,
   owner_keys : Vec<crypto::sign::PublicKey>,
   version : u64,
   previous_owner_keys : vec<crypto::sign::PublicKey>
   signature : Vec<Signature>
}

impl StructuredData {
    pub fn new(
        tag_type : u64,
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

    pub fn add_signature(&mut self,
        private_sign_key : &crypto::sign::SecretKey,
        public_sign_key: &crypto::sign::PublicKey) -> Result {}

    pub fn is_valid_successor(&self, successor: &StructuredData) -> bool {
        // see detailed discussion of logic below
    }
}
```

- `data` Routing does not parse the `data` field; it is always considered
as serialised bytes.
- `version` Routing does not (currently) attempt to resolve concurreny issues;
as such `version` is unused at routing.  Additionally routing has no knowledge
of the previously stored valid version. The version number is for the user.
- `tag_type` Routing does not attach meaning to the 8 byte `tag_type` and treats
all tag_types equal.
- the signature should sign in bytes that are concatenated
in the following order: `data`, `version`, `owner_keys` and
`previous_owner_keys` - purely as a matter of convention.

Corresponding `get` functions for all data fields are implied.

#### logic for valid succession of structured data

The structured data is recursively validated strictly on the preceding version.
That is the validity of a `StructuredData` of version `n` can only depend
on the validity of the same `StructuredData` of version `n-1`.  Here "same"
means "have the same name".  For `StructuredData` of version `n = 0` the
validity is `true` (and hence ClientManagers can charge an account to create
new valid `StructuredData_v0`).

The version number must strictly increase by one.
All structured data must be owned; transfer of ownership is discussed below.
A majority is 50%, as outlined in the parent RFC

**case : previous owner field is empty**

An empty `previous_owner_keys` vector implies that the `owner_keys`
of version `n` must match the `owner_keys` of version `n-1`.
The `previous_owner_keys` of version `n-1` is ignored.
In this case no transfer of ownership is executed.  The signatures must
be validatable with a majority of the keys of the current owners of version `n`.

**case : previous owner field is not empty**

A non-empty `previous_owner_keys` vector implies that there is an intent to
transfer ownership. In this case the `previous_owner_keys` of version `n`
must match the `owner_keys` of version `n-1`. The signatures must be
validatable with a majority of the keys of `previous_owner_keys` of version `n`.

#### efficiency consideration when validating the signatures

As outlined above, a signed vector of public keys defines a fixed order of
the public keys that will be checked to validate the signatures (see above
on previous owners or not).  On construction of a `StructuredData`
the `Vec<Signatures>` will be ordered to the fixed order defined by the
relevant `Vec<PublicKey>`.

Any validation effort will then in sequence iterate over the signatures, and
in sequence check the keys, starting from the same index.  This is an
easy algorithm for minimizing the search for matching signatures.  A better
algorithm can be suggested.

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

## Conservative approach to MessageTypes

Routing can reduce to the following message types; the main motivation
is to simplify the message handlers.  However, we will first go for
a conservative approach and extend the existing paradigm.

This requires updating `GetData`, `GetDataResponse`, `PutData`, `PutDataResponse`.
It leads to introducing `Post`, `PostResponse`, `Delete`, `DeleteResponse`.

The corresponding handlers need to be updated/written, replacing `serialised_bytes`
with `UnifiedData`.

``` rust
pub enum UnifiedData {
    PlainData,
    ImmutableData(u8),
    StructuredData(u64)
}
```

## also update with UnifiedData enumeration
This list is not exhaustive; all previously `serialised bytes`
need to be updated to a `UnifiedData`.

- `NameTypeAndId`
- `MessageAction`
- `MethodCall`
- `RoutingMembrane::Put`, `RoutingMembrane::Get`, etc
- `RoutingClient::Put`, `RoutingClient::Get`, etc
- `NodeInterface`
- `ClientInterface`

# Drawbacks

Why should we *not* do this?
(uncompleted)

# Alternatives

What other designs have been considered? What is the impact of not doing this?
(uncompleted)

# Unresolved questions

What parts of the design are still to be done?
(uncompleted)
