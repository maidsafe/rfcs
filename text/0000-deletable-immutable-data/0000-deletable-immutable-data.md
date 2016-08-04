# ImmutableData deletion support
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_vault`, `safe_launcher`
- Start Date: 04-August-2016

## Summary
This is an attempt to present the benefits of being able to delete `ImmutableData` and show a possible approach to do it.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
`ImmutableData` is considered to be a network-owned data. As such users of SAFE Network can create it but never delete it. Moreover by their very nature `ImmutableData` are immutable and this immutablility is baked into the fact that the location where `ImmutableData` are stored (also called the `name` of `ImmutableData`) is derived from the hash of its content (also referred to as the `body`). Any change in the content will consequently produce a new name and the network will store it in a different place. Also `ImmutableData` is de-duplicated, i.e. if several users upload the same `ImmutableData`, they will all be charged but the network itself will have only one primary copy of it. However there are several cases where de-duplication is very unlikely, for instance if the content was encrypted or if the content was a representation of versioned directory listing of a particular user etc. In these cases when the user updates the content, they might no longer have any need for the previous content. There are also cases where one would like to delete a previously created `ImmutableData`. Currently there is neither a mechanism nor an incentive to delete unneeded `ImmutableData`. In practice we find such accumulation of never required data highly wasteful. Each churn event must account for exchange of all such data too as the network has no knowledge of what is required and what is not. Being able to delete `ImmutableData` would be nice to have and should be incentivised.

## Detailed design
When `ImmutableData` is created, the location of its storage in the network is derived from the hash of its content (`Vec<u8>`). We can imagine an approximation as:
```rust
// Immutable data representation:
struct ImmutableData(Vec<u8>);
let immut_data = ImmutableData::new(vec![255; 10]);

// Name:
let immut_data_name = XorName::new(sha512::hash(&immut_data.0).0);

// Location on SAFE Network: stored by group of nodes close to immut_data_name

//Conceptual storage in a node:
let storage: HashMap<XorName, ImmutableData>;
```
For someone to be able to delete it there has to be some notion of ownership tied to it, otherwise `B` can delete data that was created only by `A`. We need a mechanism in place where `B` can delete data created by `A` iff `B` too had created exactly the same data. After `B` has done deleting, the data should still remain as `A` hasn't deleted it yet. Incentivising would mean that whoever deletes data gets reimbursed for playing their part in freeing up space. From this point of view, we don't want `B's` subsequent delete attempts to be legit (allowed), because `B` has already been reimbursed. When `A` finally issues a delete, a reimbursement is credited and data is finally deleted by the network.

Changing what is mentioned as a conceptual storage of `ImmutableData` in a node above can help achive this:
```rust
enum Owner {
    Users(Vec<sign::PublicKey>),
    Network,
}

struct ImmutableDataStorage {
    immut_data: ImmutableData,
    owner: Owner,
}

// New way of storage:
let new_storage: HashMap<XorName, ImmutableDataStorage>;
```
Each time someone creates the same `ImmutableData`, one's `PublicKey` get appended to the `Owner::Users` field. Similarly each delete request from a user will be considered valid if the user's key is present in the list and if so the found key will be removed from the list. Once the length of `Owner::Users` reduces to zero, the data is deleted from the network. This field must allow duplication because one user can create same `ImmutableData` multiple times. For e.g. if `A` creates same `ImmutableData` thrice, `A` can issue 3 deletes all of which will be successful and will yeild a reimbursement. The last delete will actually delete the data from the network.

There is however a downside - for popular data which might be deduplicated a large number of times, `Owner::Users` will grow enormously in size. A solution to this is that deletion of `ImmutableData` must be entail a _best-effort_ guarantee. If the deduplication rises above a limit (hence highly popular), we will consider that data to be entirely network owned and can no longer be deleted by any of its creators. At this point the storage will flip `owner` to `Owner::Network` and beyond this, `ImmutableData` can no longer be deleted and has no associated owners. The policy of reimbursement will not be applicable anymore.

This upper limit shall be called `DEDUPLICATION_LIMIT` and shall be defined as follows: _The number of unique keys in `Owner::Users` beyond which this `ImmutableData` is entirely owned by the network_.
Its value is chosen thus:
```rust
const DEDUPLICATION_LIMIT: usize = 20;
```
Once number of unique keys in `Owner::Users` is greater than `DEDUPLICATION_LIMIT`, an attempt to delete such an `ImmutableData` shall result in the following (existing) error:
```rust
// Defined in safe_network_common at the time of this RFC
MutationError::InvalidOperation
```

## Drawbacks
- A slight increase in complexity at vault level for handling of `ImmutableData`.
- The deletion (and hence the incentive of reimbursement) is not a full guarantee but a best-effort one.

## Alternatives
* Don't implement this and keep everything as is if it is provable that the network will not be impacted by large amount of unneeded data when the network itself grows to enormous sizes.
