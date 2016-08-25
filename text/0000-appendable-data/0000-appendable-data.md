# Appendable Data
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_vault`, `safe_launcher`, `routing`
- Start Date: 25-August-2016

## Summary
Facility for non-owner updation of owned data for the purposes of information exchange.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
`StructuredData` can only be modified by its owners as otherwise the vaults would invalidate the operation. However if `B`, `C`, `D` need to communicate with `A`, there is currently no easy mechanism. If it were possible for `A` to advertise some place where anyone could add information and `A` could retrieve it at will, many forms of information exchange would be immediately realisable. This calls for the need of a new fundamental type with a few new operations and we will have laid the foundation of information exchange in SAFE Network around and upon which exciting apps can be built. This is what Appendable-Data aims to provide.

## Detailed design
Some of the features, such as notions of ownership and abitlity to trasfer it etc., are directly borrowed from `StructuredData`. As such we will dive into the type-definition straight away. There shall be two kinds of this new type:
```rust
struct PubAppendableData {
    name        : XorName,
    version     : u64,
    owners      : Vec<sign::PublicKey>,
    prev_owners : Vec<sign::PublicKey>,
    filter      : (FilterType, Vec<sign::PublicKey>),
    deleted_data: HashSet<AppendedData>,
    signature   : Signature, // All the above fields
    data        : HashSet<AppendedData>, // Unsigned
}

struct PrivAppendableData {
    name        : XorName,
    version     : u64,
    owners      : Vec<sign::PublicKey>,
    prev_owners : Vec<sign::PublicKey>,
    filter      : (FilterType, Vec<sign::PublicKey>),
    encrypt_key : box_::PublicKey,
    deleted_data: HashSet<(box_::PublicKey, Vec<u8>)>,
    signature   : Signature, // All the above fields
    data        : HashSet<(box_::PublicKey, Vec<u8>)>, // Unsigned
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
    owner    : sign::PublicKey,
    signature: Signature, // All the above fields
}
```

- Both `PubAppendableData` and `PrivAppendableData` shall have max size restriction of **100 KiB**.
- `AppendedData` contains the location (pointer) of the actual data which can be any type (`Immutable`, `Structured` etc.) as identified by `DataIdentifier`. The pointer mechanism servers two purposes:
  - It keeps appended data small as it only contains a pointer to the actual data which could be colossal and fill entire 100 KiB limit by just itself.
  - The person adding the data needs to first create the actual data via `PUT` thus paying for what they create instead of merely adding data into someone else's location gratis.
- As shown all fields apart from `data` are only owner modifiable and needs to be signed by the owner/s. `data` however will be unsigned and must be modifiable by anyone who passes the `filter` criteria set by owner/s. Owner/s can set filter to either blacklist or whitelist. In case it is set to `FilterType::BlackList`, the vaults shall enforce the rule of allowing everyone but the blacklisted keys to add to `data` on subsequent updation following the change to `filter`, i.e. existing data in `data` field will not be dealt by the vaults. Similarly, if set to `FilterType::WhiteList`, no one but the the whitelisted keys will be allowed to add data.
- Simultaneous update of `data` will be dealt by a merge operation at the vaults. For e.g. if `data` was orignially empty and 3 updates from 3 different sources arrived simultaneously, then vaults would do a union operation, and the resultant appendable data would have all three updates.
- Deletion of data is done by moving the particular data in `data` field to `deleted_data` by the owner followed by the `POST` of the entire `PubAppendableData`/`PrivAppendableData`. As with any owner related modifications via `POST`, the vaults shall in this case assert the version increment.

### PubAppendableData vs PrivAppendableData
In case of `PubAppendableData`, `AppendedData` can be added as is. This means that public can be see how many of them are present and who added them. However if the owner does not want to share this information, `PrivAppendableData` caters for the required privacy. It comes with extra field called `encrypt_key` where the owner supplies the key which everyone appending data should encrypt both `AppendedData` and the actual data with. Since encyption would also imply that the owner have access to `box_::PublicKey` of the person encrypting and appending, `data` field in this case is a tuple containing encrypted `AppendedData` and sender's `box_::PublicKey`. This `box_::PublicKey` from the sender may be a part of throw-away key-pair used just for this encryption or something more permanent - it is completely upto the sender.

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
In addition to `PUT/POST/DELTE` mutating operatons, we will use a new API called `APPEND` to achive the append operations. At routing-safe_core interface this shall be:
```rust
pub fn send_append_request(data_id: DataIdentifier, data: Vec<u8>, msg_id: MessageId) -> Result<(), InterfaceError>;
```
where
- `data_id` is either `DataIdentifier::PubAppendable` or `DataIdentifier::PrivAppendable`
- `data` is
  - For `data_id == DataIdentifier::PubAppendable` : Serialised AppendedData. Vault can deserialise it and extract the sender key for filter check and signature check.
  - For `data_id == DataIdentifier::PrivAppendable`: `Serialised(Signature, sign::PublicKey, Serialised(box_::PublicKey, Encrypted(AppendedData)))`. The signature is the result of signing the other two fields. Encryption is performed using owner's `encrypt_key` and sender's `box_::SecretKey`, the public part of which is transmitted. In this case the vaults will extract the sender key for filter check and signature validation. If successful it will discard them and store only the `Serialised(box_::PublicKey, Encrypted(AppendedData))` part.

The `Authority` for append operations will always default to `Authority::NaeManager`.

## Drawbacks
- There is currently no _push_ mechanism for append operation, i.e. the owner must resort to polling (as opposed to notifications) to check if `Priv/PubAppendableData` has been updated.

## Alternatives
- None yet
