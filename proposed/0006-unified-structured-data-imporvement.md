- Feature Name: Handle Unified Structured data in conjunction with Vaults to prevent collision
- Type: Enhancement
- Related components: routing, maidsafe_types, maidsafe_vault, maidsafe_client, sentinel
- Start Date: 09-07-2015

# Summary

Use Unified Structured Data (Rfc 0000-Unified-structured-data.md) `previous_owner_keys` to only prevent collision in vaults.

# Motivation

##Why?

The field `previous_owner_keys` will be filled only during alteration of existing owners (redunction, addition and transfer) to allow vaults to re-hash a new key to avoid collision. They will solve no other purpose.

##What cases does it support?

This change supports all use of non immutable data (structured data). This covers all non `content only` data on the network and how it is handled.

##Expected outcome

Changing the purpose of field `previous_owner_keys` simplifies code and reduces logical complexity during Structured Data Updates and ownership-transfers. This Rfc shows the only purpose of this field in improving vault logic in future to avoid Structured Data Collisions even for the same `location` (64 byte network address for the data) as vaults will utilise this field to make the storage key unique even though `location` wasn't unique.

# Detailed design

##StructuredData

```
struct StructuredData {
    tag_type           : u64,
    identifier         : crypto::hash::sha512::Digest,
    version            : u64,                         // mutable - incrementing (deterministic) version number
    data               : Vec<u8>,                     // mutable - in many cases this is encrypted
    owner_keys         : vec<crypto::sign::PublicKey> // mutable - n * 32 Bytes (where n is number of owners)
    previous_owner_keys: vec<crypto::sign::PublicKey  // mutable - m * 32 Bytes (where n is number of owners)
    signatures         : Vec<crypto::sign::Signature> // mutable - detached signature of all the `mutable` fields (barring itself) by creating/updating owners
}
```

Fixed (immutable fields)
- tag_type
- identifier

##Flow

The following shows Creation, Updation and Transfer-of-ownership of Structured Data:

**step 0:**
Client-A sends:
```
StructuredData {
    other_fields: Value-00,
    owner_keys: A-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 1:**
Logic: Since `previous_owner_keys` in incoming data contains nothing, use `key` = `(sha512(owner_keys), name)` for look-up.
Vault receives for the first time. So Just Stores it:
```
Key = (sha512(A-PublicKey), name)
Value = StructuredData {
    other_fields: Value-00,
    owner_keys: A-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 2:**
Client-A updates it:
```
StructuredData {
    other_fields: Value-11,
    owner_keys: A-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 3:**
Logic: Since `previous_owner_keys` in incoming data contains nothing, use `key` = `(sha512(owner_keys), name)` for look-up.
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 1 to verify signature of data in step 2. If valid replace previous data and store new one:
```
Key = (sha512(A-PublicKey), name)
Value = StructuredData {
    other_fields: Value-11,
    owner_keys: A-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 4:**
Client-A wants to transfer ownership of Structured Data to Client-B:
```
StructuredData {
    other_fields: Value-22,
    owner_keys: B-PublicKey,
    previous_owner_keys: A-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 5:**
Logic: Since `previous_owner_keys` in incoming data contains something, use `key` = `(sha512(previous_owner_keys), name)` for look-up. Calculate a new `key` = `(sha512(owner_keys), name)` if the update is legal as checked below (as usual).
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 3 to verify signature of data in step 4. If valid replace previous data and store new one:
```
Key = (sha512(B-PublicKey), name)
Value = StructuredData {
    other_fields: Value-22,
    owner_keys: B-PublicKey,
    previous_owner_keys: A-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 6:**
Client-A tries to update illegally:
```
StructuredData {
    other_fields: Value-22,
    owner_keys: Anything,
    previous_owner_keys: Anything/NONE,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 7:**
If look-up succeeds, Vault will use owner field from **step 5** == `B-PublicKey` and signature verification fails. So it will not update anything.
 
**step 8:**
Client-B tries to update:
```
StructuredData {
    other_fields: Value-33,
    owner_keys: B-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by B-PrivateKey,
}
```
 
**step 9:**
Logic: Since `previous_owner_keys` in incoming data contains nothing, use `key` = `(sha512(owner_keys), name)` for look-up.
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 5 to verify signature of data in step 8. If valid replace previous data and store new one:
```
Key = (sha512(B-PublicKey), name)
Value = StructuredData {
    other_fields: Value-33,
    owner_keys: B-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by B-PrivateKey,
}
```

# Drawbacks
None identified yet.

# Alternatives
None yet.

# Unresolved questions
None yet.
