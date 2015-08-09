- Feature Name: Handle Unified Structured data in conjunction with Vaults to prevent collision
- Type: Enhancement
- Related components: routing, maidsafe_types, maidsafe_vault, maidsafe_client, sentinel
- Start Date: 09-07-2015

# Summary

Use Unified Structured Data (Rfc 0000-Unified-structured-data.md) `previous_owner_keys` to transfer ownership and to prevent collision in vaults.

# Motivation

##Why?

The field `previous_owner_keys` will be filled only during alteration of existing owners (redunction, addition and transfer) to allow vaults to re-hash a new key to avoid collision. The overall effect will be to reduce collision of PUTs for Structured Data in Vaults by a good enough factor, as the identifier chosen by the user can be SHA512 of a string and user chosen strings may collide severly.

##What cases does it support?

This change supports all use of non immutable data (structured data). This covers all non `content only` data on the network and how it is handled.

##Expected outcome

Changing the purpose of field `previous_owner_keys` simplifies code and reduces logical complexity during Structured Data Updates and ownership-transfers. During regular updates (POSTs which don't involve ownership transfers) this will reduce the size of StructuredData packet as `owners` field will not be needed to be duplicted and filled in the `previous-owners` field (which) in this case will be blank. Also, and farther-reaching, this Rfc shows the purpose of this field in improving vault logic in future to avoid Structured Data Collisions even for the same `location` (64 byte network address for the data) as vaults will utilise this field to make the storage key unique even though `location` wasn't unique.

# Detailed design

##StructuredData

```
struct StructuredData {
    tag_type           : u64,
    identifier         : NameType,                    // struct NameType([u8; 64]);
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
Logic: Since `previous_owner_keys` in incoming data contains nothing, use `key` = `(sha512(owner_keys), Name)` for look-up.
Vault receives for the first time. So Just Stores it:
```
Key = (sha512(A-PublicKey), Name)
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
Logic: Since `previous_owner_keys` in incoming data contains nothing, use `key` = `(sha512(owner_keys), Name)` for look-up.
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 1 to verify signature of data in step 2. If valid replace previous data and store new one:
```
Key = (sha512(A-PublicKey), Name)
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
Logic: Since `previous_owner_keys` in incoming data contains something, use `key` = `(sha512(previous_owner_keys), Name)` for look-up. Calculate a new `key` = `(sha512(owner_keys), Name)` if the update is legal as checked below (as usual).
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 3 to verify signature of data in step 4. If valid replace previous data and store new one:
```
Key = (sha512(B-PublicKey), Name)
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
Logic: Since `previous_owner_keys` in incoming data contains nothing, use `key` = `(sha512(owner_keys), Name)` for look-up.
Vault will check if new data is sent by valid owner. Use `owner_keys` from the stored data in step 5 to verify signature of data in step 8. If valid replace previous data and store new one:
```
Key = (sha512(B-PublicKey), Name)
Value = StructuredData {
    other_fields: Value-33,
    owner_keys: B-PublicKey,
    previous_owner_keys: NONE,
    signatures: Signed by B-PrivateKey,
}
```
## Change in the Definition of GETs
The above shows the cases for PUTs, POSTs and DELETEs. There will be a change in the signature of GETs to achieve the above no-collision scheme. GETs for StructuredData need to be changed from:
```
GET(Name, DataRequest::StructuredData(u64))
```
to
```
GET(Name, DataRequest::StructuredData(SHA512(PublicKeys)))
```
For ordinary cases where `data-PUTers` themselves are fetching thier data (majority of clients) this does not mean much as the client engine will inject this silently as it already has the knowledge of user public-keys. The only time it does not have the knowledge is during the fetching of the session packet. The solution for this is easy as login packet has a dedicated maidasafe-type-tag and vaults can recognise it and make special allowance for it to be stored with `Key = Name` instead of `Key = (sha512(PublicKeys), Name)` as in above.

#Drawbacks
For other cases where the retrieval is done for the data PUT by others, the GETer must know the PublicKey of the original PUTer. An example of this will be the fetching of Dns Packets by a safe-url-parser (could be a browser). This may be seen as an advantage instead of disadvantage though as explained thus. Since there is no retriction for taking a Dns name in the SAFE-network, a user can very easily obtain well known Dns names like `pepsico.com` preventing the legit organisation to take it later. However with the schema in thi rfc, the legit organisation can easily obtain the same Dns name for it because it will have got the uniqueness in the vaults due to `sha512` of its PublicKey. This PublicKey will be the one which is distributed to all browsers and thus safe-url-parser will land on the legitimate page instead of the pretender's.

Since Dns Packets also have special type-tags, the vaults can recognise them. There could be ordinary ones (with TAG = X) where the storage is `Key = Name` for which there would be collisions and special ones (with TAG = Y) where the storage would be `Key = (sha512(PublicKey), Name)`. The safe-browser would land on the correct destination depending on whether TAG = X or TAG = Y was demanded. This is beyond the scope of this Rfc and can be discussed in a separate one.

# Alternatives
None yet.

# Unresolved questions
Working of safe-protocol for browsers as briefly discussed in Drawbacks section.
