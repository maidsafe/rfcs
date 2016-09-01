# Appendable Data
- Status: active
- Type: feature
- Related components: `safe_core`, `safe_vault`, `safe_launcher`, `routing`
- Start Date: 25-August-2016
- Discussion: https://forum.safedev.org/t/rfc-38-appendable-data/72

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
    pub name                     : XorName,
    pub version                  : u64,
    pub current_owner_keys       : Vec<sign::PublicKey>,
    pub previous_owner_keys      : Vec<sign::PublicKey>,
    pub filter                   : Filter,
    pub deleted_data             : BTreeSet<AppendedData>,
    pub previous_owner_signatures: Vec<Signature>, // All the above fields
    pub data                     : BTreeSet<AppendedData>, // Unsigned
}

impl PubAppendableData {
    // Required member function
    pub fn add_signature(&mut self, secret_key: &sign::SecretKey);
}

struct PrivAppendableData {
    pub name                     : XorName,
    pub version                  : u64,
    pub current_owner_keys       : Vec<sign::PublicKey>,
    pub previous_owner_keys      : Vec<sign::PublicKey>,
    pub filter                   : Filter,
    pub encrypt_key              : box_::PublicKey,
    pub deleted_data             : BTreeSet<PrivAppendedData>,
    pub previous_owner_signatures: Vec<Signature>, // All the above fields
    pub data                     : BTreeSet<PrivAppendedData>, // Unsigned
}

impl PrivAppendableData {
    // Required member function
    pub fn add_signature(&mut self, secret_key: &sign::SecretKey);
}
```
where
```rust
pub enum Filter {
    BlackList(BTreeSet<sign::PublicKey>),
    WhiteList(BTreeSet<sign::PublicKey>),
}

// Outer cover discarded by vaults, after filter check and signature validation where applicable.
// Only data used
pub enum AppendWrapper {
  Pub {
    append_to: XorName,
    data     : AppendedData,
  }
  Priv {
    append_to: XorName,
    data     : PrivAppendedData,
    sign_key : sign::PublicKey,
    signature: Signature, // All the above fields
  }
}

impl AppendWrapper {
    pub fn new_pub(append_to: XorName, data: AppendedData) -> Self;
    pub fn new_priv(append_to: XorName,
                    data: PrivAppendedData,
                    sign_pair: (&sign::PublicKey, &sign::SecretKey)) -> Self;
}

struct AppendedData {
    pub pointer  : DataIdentifier, // Pointer to actual data
    pub sign_key : sign::PublicKey,
    pub signature: Signature, // All the above fields
}

impl AppendedData {
    // Required member function
    pub fn sign(&mut self, secret_key: &sign::SecretKey);
}

struct PrivAppendedData {
    pub encrypt_key: box_::PublicKey, // Recommended to be a part of a throwaway keypair
    pub encrypted_appeneded_data : Vec<u8>, // Encrypted AppendedData
}
```

- Both `PubAppendableData` and `PrivAppendableData` shall have max size restriction of **100 KiB**.
- `AppendedData` contains the location (pointer) of the actual data which can be any type (`Immutable`, `Structured`, etc.) as identified by `DataIdentifier`. The pointer mechanism will keep appended data small as it only contains a pointer to the actual data which could be colossal and fill entire 100 KiB limit by just itself.
- As shown, all fields of `Pub/PrivAppendableData` apart from `Pub/PrivAppendableData::data` are only owner modifiable and need to be signed by the owner(s). `Pub/PrivAppendableData::data` however will be unsigned and shall be modifiable by anyone who passes the `filter` criteria set by owner(s). Owner(s) can set `filter` to either blacklist or whitelist. In case it is set to `Filter::BlackList`, the vaults shall enforce the rule of allowing everyone but the blacklisted keys to add to `data` on subsequent updates following the change to `filter`, i.e. existing data in `data` field will not be dealt by the vaults. Similarly, if set to `Filter::WhiteList`, no one but the the whitelisted keys will be allowed to add data.
- Simultaneous updates of `Pub/PrivAppendableData::data` will be dealt with by a merge operation at the vaults. For e.g. if `Pub/PrivAppendableData::data` was originally empty and 3 updates from 3 different sources arrived simultaneously, then vaults would do a union operation, and the resultant appendable data would have all three updates.
- Deletion of data is done by moving the particular data in `Pub/PrivAppendableData::data` field to `Pub/PrivAppendableData::deleted_data` by the owner followed by the `POST` of the entire `Pub/PrivAppendableData`. As with any owner related modifications via `POST`, the vaults shall in this case assert the version increment. It is important to delete in this manner otherwise churn can bring back the deleted data in some cases. When data is moved from `Pub/PrivAppendableData::data` to `Pub/PrivAppendableData::delete_data`, vaults will ensure that any merge operation that tries to put data back into `Pub/PrivAppendableData::data` while it also resides in `Pub/PrivAppendableData::deleted_data` shall be ignored. Emptying `Pub/PrivAppendableData::deleted_data` itself can be done by the user by similar `POST` after some period of time which though is unspecified, but should be safe in excess of 20 min or so (in case it's a heavily churning network at that point of time or much earlier otherwise).
- `PrivAppendedData::encrypted_appeneded_data` is encrypted `AppendedData` using `PrivAppendableData::encrypt_key` and sender's `box_::SecretKey`, the public part of which is `PrivAppendedData::encrypt_key`. Owner(s) of `PrivAppendableData` will use their `box_::SecretKey` and `PrivAppendedData::encrypt_key` to decrypt data into `AppendedData` and then retrive actual data from `AppendedData::pointer` which will be usually encrypted in the exact same way.

### `PubAppendableData` vs `PrivAppendableData`
In the case of `PubAppendableData`, `AppendedData` can be added as is. This means that anyone can see what has been added and who added them. However if the owner does not want to share this information, `PrivAppendableData` caters for the required privacy. It comes with an extra field called `encrypt_key` where the owner supplies the key with which everyone appending data shall encrypt `AppendedData` and the actual data pointed to by it. Since encryption would also imply that the owner has access to `box_::PublicKey` of the person encrypting and appending, `data` field in this case is `PrivAppendedData` containing encrypted `AppendedData` and sender's `box_::PublicKey`. This `box_::PublicKey` from the sender may be a part of a throw-away key-pair used just for this encryption or something more permanent - it is completely up to the sender.

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
pub fn send_append_request(dst: Authority, wrapper: AppendWrapper, msg_id: MessageId) -> Result<(), InterfaceError>;
```
where
- `dst`: While `routing` expects a generic `Authority`, `safe_core` and `safe_vault` will use the `Authority::MaidManager` authority of the sender to facilitate append operations being charged for the sender.
- `wrapper` helps route message to either `Priv/PubAppendableData` where it is required to append the data.
  - For `wrapper == AppendWrapper::Pub { append_to, data }`: Operation is straight forward. Once routed to `append_to`, vaults will simply throw away the wrapper and store `data` after signature verification and filter check using  `data.signature` and `data.sign_key`.
  - For `wrapper == AppendWrapper::Priv { append_to, data, sign_key, signature }`: Once routed to `append_to`, vaults will perform filter check and signature using `sign_key` and `signature` and then discard it storing `data == PrivAppendedData`.

### Deletion
When onwer `POSTs` a `Pub/PrivAppendableData` with some entires in `Pub/PrivAppendableData::data` moved to `Pub/PrivAppendableData::deleted_data`, vaults (after all the usual version bump, signature etc. checks) shall do a merge (union) of current `Pub/PrivAppendableData::data` they are holding and `Pub/PubAppendableData::data` in the new version they are about to store, purging anything in new version's `Pub/PrivAppendableData::deleted_data`. This makes sure that the new entries that arrived while the owner was in the process of `POSTing` are not discarded.

## Drawbacks
- There is currently no _push_ mechanism for the append operation, i.e. the owner must resort to polling (as opposed to notifications) to check if `Priv/PubAppendableData` has been updated.
- There is no way for owner to actually blacklist a spammer in case of `PrivAppendableData`. This is because the outer shell which is discarded by the vaults after verification contains the `sign::PublicKey` which the vaults actually use for filter checks. However what owner sees as `sign::PublicKey` after decrypting the data might not be the same and if owner blacklists this key, there isn't anything that is going to happen because it's the outer key that was malacious. The vaults discard this outer shell for the sake of anonymity preservation (i.e. do not want sender to be traceable).
