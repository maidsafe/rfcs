- Feature Name: Improve Unified Structured  data 
- Type: Enhancement
- Related components: routing, maidsafe_types, maidsafe_vault, maidsafe_client, sentinel
- Start Date: 09-07-2015

# Summary

Make Unified Structured Data (Rfc 0000-Unified-structured-data.md) work without the field `previous_owner_keys`.

# Motivation

##Why?

The field `previous_owner_keys` causes code complexity and cofusion. Unified Structured Data can be made to work without it.

##What cases does it support?

This change supports all use of non immutable data (structured data). This covers all non `content only` data on the network and how it is handled.

##Expected outcome

Removing the field `previous_owner_keys` simplifies code and reduces logical complexity during Structured Data Updates and ownership-transfers. This might further simplify vault logic in future to avoid Structured Data Collisions even for the same `location` (64 byte network address for the data) as vaults will now have to look only for a single field (`owners`) and track its changes to make the storage key unique even though `location` wasn't unique.

# Detailed design

##StructuredData

```
struct StructuredData {
    tag_type   : u64,
    identifier : crypto::hash::sha512::Digest,
    version    : u64,                         // mutable - incrementing (deterministic) version number
    data       : Vec<u8>,                     // mutable - in many cases this is encrypted
    owner_keys : vec<crypto::sign::PublicKey> // mutable - n * 32 Bytes (where n is number of owners)
    signatures : Vec<crypto::sign::Signature> // mutable - detached signature of all the `mutable` fields (barring itself) by owners
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
    signatures: Signed by A-PrivateKey,
}
```
 
**step 1:**
Vault receives for the first time. So Just Stores it:
```
StructuredData {
    other_fields: Value-00,
    owner_keys: A-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 2:**
Client-A updates it:
```
StructuredData {
    other_fields: Value-11,
    owner_keys: A-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 3:**
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 1 to verify signature of data in step 2. If valid replace previous data and store new one:
```
StructuredData {
    other_fields: Value-11,
    owner_keys: A-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 4:**
Client-A wants to transfer ownership of Structured Data to Client-B:
```
StructuredData {
    other_fields: Value-22,
    owner_keys: B-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 5:**
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 3 to verify signature of data in step 4. If valid replace previous data and store new one:
```
StructuredData {
    other_fields: Value-22,
    owner_keys: B-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 6:**
Client-A tries to update illegally:
```
StructuredData {
    other_fields: Value-22,
    owner_keys: A-PublicKey,
    signatures: Signed by A-PrivateKey,
}
```
 
**step 7:**
Vault will use owner field from step-5 == B-PublicKey and signature verification fails. So it will not update anything.
 
**step 8:**
Client-B tries to update:
```
StructuredData {
    other_fields: Value-33,
    owner_keys: B-PublicKey,
    signatures: Signed by B-PrivateKey,
}
```
 
**step 9:**
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 5 to verify signature of data in step 8. If valid replace previous data and store new one:
```
StructuredData {
    other_fields: Value-33,
    owner_keys: B-PublicKey,
    signatures: Signed by B-PrivateKey,
}
```

# Drawbacks
None identified yet.

# Alternatives
None yet.

# Unresolved questions
None yet.
