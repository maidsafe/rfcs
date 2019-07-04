# Unpublished ImmutableData

- Status: proposed
- Type: enhancement
- Related components: safe_client_libs, safe_vault
- Start Date: 16-05-2019
- Discussion: https://safenetforum.org/t/rfc-55-unpublished-immutabledata/28621

## Summary

This document describes how to enhance `ImmutableData` to make it an unpublished or published, with the difference that unpublished can be deleted. The published `ImmutableData` is the normal `ImmutableData` we have just now. There are no changes to that. This RFC is only for the extension to such a data to allow an `Unpublished` kind of it to be supported by the Network.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- `Unpublished` data is any data that is only get-able by the uploader and not by general public.
- `Published` data is any data that is available for the public - i.e., any user of SAFE Network can fetch it.

## Motivation
There are many a times when we create `ImmutableData` to store private content. Sometimes these contents might have been pre-encrypted so that any chances of having de-duplication due to self-encryption is very minimal. Also the users might not choose to utilise self-encryption and just use custom algorithms. In such cases it makes sense if the Network allowed deletion of `ImmutableData` instead of it storing many of them which might no longer be required even by the original uploader. However all such data will be regarded as unpublished as opposed to current `ImmutableData` which will be categorised as published and cannot be taken out once put.

## Assumptions
- As mentioned in the [RFC on MutableData enhancements](https://github.com/maidsafe/pre-rfc/blob/master/vault/mutable-data-enhancement.md), the `GETs` go through `ManidManagers` - so `clients <-> MaidManagers <-> DataManagers`
- The replay attacks are circumvented by the (currently upcoming) safe-coin RFC.

## Detailed design

### UnpublishedImmutableData

- Unlike `ImmutableData` this SHALL NOT be deduplicated - so `PUTs` to the same location will result in conflict error.
- The Network SHALL enforce that the `GETs` are only allowed by the owner(s). For this we SHALL use the special `OwnerGet` RPC
- Since replay attacks are thwarted by safecoin payments and transaction history, the Network SHALL allow complete deletion of this data by `DELETE` operation. This `DELETE` should be directed in the same way mutable-data deletes are directed.
- There SHALL only be one time update of owner(s), which happens during the creation, and this type is non-transferable. Changing the owners would mean new keys which would result in change of the location where the data is, which would be meaningless. Hence no changes to the owner field is allowed once the data is created.
- Published and unpublished immutable data MUST use the same XOR namespace. I.e., if a user stores an `UnpublishedImmutableData` at a XOR address X and another user tries to store a `PublishedImmutableData` at the same XOR address there will be a conflict and an error will be returned. When trying to fetch an `UnpublishedImmutableData` at an address X, if a `PublishedImmutableData` is found an `UnexpectedData` error is thrown.

```rust
pub struct UnpublishedImmutableData {
    /// Contained ImmutableData
    data: Vec<u8>,
    /// Contains a set of owners of this data. DataManagers enforce that a
    /// DELETE or OWNED-GET type of request is coming from the
    /// MaidManager Authority of the owners.
    owners: BLS-PublicKey,
}

impl UnpublishedImmutableData {
    /// Name
    pub fn name(&self) -> XorName {
        let c = CONCAT(HASH(self.data), self.owner);
        HASH(c)
    }
}
```
- In summary, the only RPCs allowed for such a data type SHALL be `PUT` (to create), `OWNED-GET` (to retrieve) and `DELETE`, all done by the owner(s).


## Drawbacks

## Alternatives

## Unresolved questions
