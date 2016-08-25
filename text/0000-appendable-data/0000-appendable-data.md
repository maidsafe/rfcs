# Appendable Data
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_vault`, `safe_launcher`, `routing`
- Start Date: 25-August-2016

## Summary
Facility for non-owner updates of owned data for the purposes of information exchange.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
`StructuredData` can only be modified by its owners as otherwise the vaults would invalidate the operation. However if `B`, `C`, `D` need to communicate with `A`, there is currently no easy mechanism. If it were possible for `A` to advertise some place where anyone could add information and `A` could retrieve it at will, many forms of information exchange would be immediately realisable. This calls for the need of a new fundamental type with a few new operations and we will have laid the foundation of information exchange in SAFE Network around and upon which exciting apps can be built. This is what Appendable-Data aims to provide.

## Detailed design
Some of the features, such as notions of ownership and ability to transfer it etc., are directly borrowed from `StructuredData`. As such we will dive into the type-definition straight away. There shall be two kinds of this new type:
```rust
struct PubAppendableData {
    name                     : XorName,
    version                  : u64,
    current_owner_keys       : Vec<sign::PublicKey>,
    previous_owner_keys      : Vec<sign::PublicKey>,
    filter                   : (FilterType, Vec<sign::PublicKey>),
    deleted_data             : HashSet<AppendedData>,
    previous_owner_signatures: Signature, // All the above fields
    data                     : HashSet<AppendedData>, // Unsigned
}

struct PrivAppendableData {
    name                     : XorName,
    version                  : u64,
    current_owner_keys       : Vec<sign::PublicKey>,
    previous_owner_keys      : Vec<sign::PublicKey>,
    filter                   : (FilterType, Vec<sign::PublicKey>),
    encrypt_key              : box_::PublicKey,
    deleted_data             : HashSet<(box_::PublicKey, Vec<u8>)>,
    previous_owner_signatures: Signature, // All the above fields
    data                     : HashSet<(box_::PublicKey, Vec<u8>)>, // Unsigned
}
```
where
```rust
enum FilterType {
    BlackList,
    WhiteList,
}

struct AppendedData {
    pointer  : DataIdentifier, // Pointer to actual data
    sign_key : sign::PublicKey, // The sender
    signature: Signature, // All the above fields
}
```

- Both `PubAppendableData` and `PrivAppendableData` shall have max size restriction of **100 KiB**.
- `AppendedData` contains the location (pointer) of the actual data which can be any type (`Immutable`, `Structured`, etc.) as identified by `DataIdentifier`. The pointer mechanism will keep appended data small as it only contains a pointer to the actual data which could be colossal and fill entire 100 KiB limit by just itself.
- As shown, all fields apart from `data` are only owner modifiable and need to be signed by the owner(s). `data` however will be unsigned and must be modifiable by anyone who passes the `filter` criteria set by owner(s). Owner(s) can set `filter` to either blacklist or whitelist. In case it is set to `FilterType::BlackList`, the vaults shall enforce the rule of allowing everyone but the blacklisted keys to add to `data` on subsequent updates following the change to `filter`, i.e. existing data in `data` field will not be dealt by the vaults. Similarly, if set to `FilterType::WhiteList`, no one but the the whitelisted keys will be allowed to add data.
- Simultaneous updates of `data` will be dealt with by a merge operation at the vaults. For e.g. if `data` was originally empty and 3 updates from 3 different sources arrived simultaneously, then vaults would do a union operation, and the resultant appendable data would have all three updates.
- Deletion of data is done by moving the particular data in `data` field to `deleted_data` by the owner followed by the `POST` of the entire `PubAppendableData`/`PrivAppendableData`. As with any owner related modifications via `POST`, the vaults shall in this case assert the version increment. It is important to delete in this manner otherwise churn can bring back the deleted data in some cases. When data is moved from `data` to `delete_data`, vaults will ensure that any merge operation that tries to put data back into `data` while it also resides in `deleted_data` shall be ignored. Emptying `deleted_data` itself can be done by the user by similar `POST` after some period of time which though is unspecified, but should be safe in excess of 20 min or so (in case it's a heavily churning network at that point of time).

### `PubAppendableData` vs `PrivAppendableData`
In the case of `PubAppendableData`, `AppendedData` can be added as is. This means that anyone can see what has been added and who added them. However if the owner does not want to share this information, `PrivAppendableData` caters for the required privacy. It comes with an extra field called `encrypt_key` where the owner supplies the key with which everyone appending data should encrypt both `AppendedData` and the actual data. Since encryption would also imply that the owner has access to `box_::PublicKey` of the person encrypting and appending, `data` field in this case is a tuple containing encrypted `AppendedData` and sender's `box_::PublicKey`. This `box_::PublicKey` from the sender may be a part of a throw-away key-pair used just for this encryption or something more permanent - it is completely up to the sender.

## Implementation
### Extending Data and DataIdentifier
`Data` and `DataIdentifier` shall encompass these new types as:
```rust
enum Data {
    // Previously present
    Structured(StructuredData),
    Immutable(ImmutableData),
    Plain(PlainData),

    // Newly added
    PubAppendable(PubAppendableData),
    PrivAppendable(PrivAppendableData),
}

enum DataIdentifier {
    // Previously present
    Structured(XorName, u64),
    Immutable(XorName),
    Plain(XorName),

    // Newly added
    PubAppendable(XorName),
    PrivAppendable(XorName),
}
```

### APPEND API
In addition to the `PUT/POST/DELETE` mutating operations, we will add an `APPEND` operation to the API. At the routing-safe_core interface this shall be:
```rust
pub fn send_append_request(data_id: DataIdentifier, data: Vec<u8>, msg_id: MessageId) -> Result<(), InterfaceError>;
```
where
- `data_id` is either `DataIdentifier::PubAppendable` or `DataIdentifier::PrivAppendable`
- `data` is
  - For `data_id == DataIdentifier::PubAppendable`: Serialised `AppendedData`. Vault can deserialise it and extract the sender key for filter check and signature check.
  - For `data_id == DataIdentifier::PrivAppendable`: `Serialised(Signature, sign::PublicKey, Serialised(box_::PublicKey, Encrypted(AppendedData)))`. The signature is the result of signing the other two fields. Encryption is performed using the owner's `encrypt_key` and the sender's `box_::SecretKey`, the public part of which is transmitted. In this case the vaults will extract the sender key for filter check and signature validation. If successful it will discard them and store only the `Serialised(box_::PublicKey, Encrypted(AppendedData))` part.

The `Authority` for append operations will always default to `Authority::NaeManager`.

## Drawbacks
- There is currently no _push_ mechanism for the append operation, i.e. the owner must resort to polling (as opposed to notifications) to check if `Priv/PubAppendableData` has been updated.
- Type erasure into `Vec<u8>` for `PrivAppendableData` and reconstruction at vaults via specific parsing logic might not be ideal way to approach this.
- There is no way for owner to actually blacklist a spammer in case of `PrivAppendableData`. This is because the outer shell which is discarded by the vaults after verification contains the `sign::PublicKey` which the vaults actually use for filter checks. However what owner sees as `sign::PublicKey` after decrypting the data might not be the same and if owner blacklists this key, there isn't anything that is going to happen because it's the outer key that was malacious. The vaults discard this outer shell for the sake of anonymity preservation (i.e. do not want sender to be traceable).

## Alternatives
### Interface
The following interface
```rust
pub fn send_append_request(data_id: DataIdentifier, data: Vec<u8>, msg_id: MessageId) -> Result<(), InterfaceError>;
```
is different from all other mutation-operation interface which look like:
```rust
pub fn send_post_request(dest: Authority, data: Data, msg_id: MessageId) -> Result<(), InterfaceError>;
```
To make it uniform and provide concrete types instead of type-erasure to `Vec<u8>` in case of `PrivAppendableData` we could divide `AppendedData` as follows:
```rust
// Outer cover discarded by vaults; only `data` used
struct PubAppendWrapper {
    append_to: DataIdentifier,
    data     : PubAppendedData,
}

// Outer cover discarded by vaults after filter check and signature validation.
// Only `data` used
struct PrivAppendWrapper {
    append_to: DataIdentifier,
    data     : PrivAppendedData,
    sign_key : sign::PublicKey,
    signature: Signature, // All the above fields
}

struct PubAppendedData {
    pointer  : DataIdentifier, // Pointer to actual data
    sign_key : sign::PublicKey,
    signature: Signature, // All the above fields
}

struct PrivAppendedData {
    msg_encrypt_key: box_::PublicKey,
    encrypted_data : Vec<u8>, // Encrypted PubAppendedData
}
```
and make them known via the usual `Data` interface:
```rust
enum Data {
    // Previously present
    Structured(StructuredData),
    Immutable(ImmutableData),
    Plain(PlainData),

    // Newly added
    PubAppendable(PubAppendableData),
    PrivAppendable(PrivAppendableData),
    PubAppendWrapper(PubAppendWrapper),
    PrivAppendWrapper(PrivAppendWrapper),
}
```

Now the function signatures and concept for all mutation would be uniform:
```rust
pub fn send_append_request(dest: Authority, data: Data, msg_id: MessageId) -> Result<(), InterfaceError>;
```
and routing location can be obtained via usual methods like `Data::name()` etc. as for `StructuredData` and `ImmutableData`. If we decide we want appends to be chargable and want to route this via `MaidManager`s we can even do this by specifying the `Authority` just like we do for `PUTs` of `StructuredData` and hence we don't sacrifice the flexibility we already have.

We would change the `Pub/PrivAppendableData` as:
```rust
struct PubAppendableData {
    name                     : XorName,
    version                  : u64,
    current_owner_keys       : Vec<sign::PublicKey>,
    previous_owner_keys      : Vec<sign::PublicKey>,
    filter                   : (FilterType, Vec<sign::PublicKey>),
    deleted_data             : HashSet<PubAppendedData>,
    previous_owner_signatures: Signature, // All the above fields
    data                     : HashSet<PubAppendedData>, // Unsigned
}

struct PrivAppendableData {
    name                     : XorName,
    version                  : u64,
    current_owner_keys       : Vec<sign::PublicKey>,
    previous_owner_keys      : Vec<sign::PublicKey>,
    filter                   : (FilterType, Vec<sign::PublicKey>),
    encrypt_key              : box_::PublicKey,
    deleted_data             : HashSet<PrivAppendedData>,
    previous_owner_signatures: Signature, // All the above fields
    data                     : HashSet<PrivAppendedData>, // Unsigned
}
```
The overhead in this approach is that 3 new types have been introduced.
