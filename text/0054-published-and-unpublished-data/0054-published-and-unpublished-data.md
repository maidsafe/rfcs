# Published and Unpublished DataType

- Status: active
- Type: enhancement
- Related components: safe_client_libs, safe_vault
- Start Date: 16-05-2019
- Discussion: https://safenetforum.org/t/rfc-54-published-and-unpublished-datatype/28620

## Summary

This document describes how to enhance the data types to allow the network to store Unpublished data via the `MutableData` type, or Unpublished or Published data via the `AppendOnlyData` type, and when these data types shall be used.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

### Published Data

Published data refers to the content that is published (made available) for everyone. For example: websites, blogs, or research papers which anyone can fetch from the network and read without requiring any permission. For such public content, it becomes important to retain a history of changes. History MUST not be allowed to be tampered with and the published data MUST remain forever.

The `AppendOnly` data type is pivotal for data perpetuity, because it ensures the published versions are always available unlike the default behaviour of `MutableData` where the value can be overwritten. This is central to prevent censorship of information.

Data perpetuity is one of the fundamentals of the SAFE Network, which ensures the users of the network SHALL be able to store published data in perpetuity.

However, only the owner(s) SHALL be able to append the changes based on the permission.

### Unpublished data

Not all the data is desired to be made public. Personal data or organisations' data stored on the network is not supposed to be accessed by everyone. Since the data is not published for everyone, this is called `UnpublishedData`. Only the owner(s) SHALL be able to access and manage the data based on the permission.

The network should also be able to provide the flexibility for the users/developers to create private data which can be versioned or mutable based on their needs.

#### Private Data

Users should be able to store private `UnpublishedData` on the network which is not accessible by anyone else.

#### Shared Data

Users should be able to store `UnpublishedData` on the network and share it with a closed group. The user should be able to give permissions like read, write, append based on the use case. For example, a collaborative document which is meant to be worked on within a closed group.

## Detailed design

Data types allowing mutations SHALL enforce that, once published, only appending new entries MUST be allowed, while entries in the private/shared data can be rewritten. Following are the considerations to enhance the mutable data types:
- `Unpublished` data MUST allow either complete mutability—overwrite of existing data (`MutableData`)—OR append only operations (`AppendOnlyData`).
- We don't need the version for preventing replay attacks, however explicitly sequencing all mutations is still an option provided for clients to allow them to avoid dealing with conflicting mutations.
- We shall sub-divide the existing `MutableData` data as follows:

    - **Complete Mutability** (`MutableData`):
        - This SHALL be very much like the current set-up (please refer to the [MutableData RFC](https://github.com/maidsafe/rfcs/blob/master/text/0047-mutable-data/0047-mutable-data.md) for details).
        - This kind SHALL only be gettable by the owner or the apps authorised by the owner. This MUST be done at the DataManagers who SHALL enforce that such a GET request be signed either by the owner or one of the Public Keys in the `Permissions` field (so that the apps can retrieve it on behalf of the owner).
        - To differentiate this kind of a `GET`, the implementation MAY give the RPC a more explicit name of `OwnerGet`.
        - Complete deletion of the key should be possible.
        - This data type is further sub-divided into two categories, `Sequenced` and `Unsequenced`.
          For `Sequenced` MutableData the client MUST specify the next version number of a value while modifying/deleting keys.
          Similarly, while modifying the Mutable Data shell (permissions, ownership, etc.), the next version number MUST be passed.
          For `Unsequenced` MutableData the client does not have to pass version numbers for keys, but it still MUST pass the next version number while modifying the Mutable Data shell.
    - **Append Only** (`AppendOnlyData`):
        - DELETE or EDIT operations SHALL NOT be permitted by the vaults on the entries of `AppendOnlyData`
        - The only available RPC for data manipulation, after it's been created, SHALL be an APPEND RPC for appending an entry.
        - The Value for the entry SHALL not have any version field as once published it cannot be deleted or overwritten in any way.
        - Any metadata field SHALL be treated similarly. E.g., `Permissions` field SHALL be a record of all permission changes and the latest one SHALL be the one that vaults take into account.
        - We sub-categorise this data as `Published` and `Unpublished`.
          `Published` data is ordinarily fetch-able by normal `GET`s by anyone.
          `Unpublished` data is only fetch-able by the owner(s) or the authorised keys. If the client wants to publish an `UnpublishedAppendOnlyData`, they need to reupload it as a new `PublishedAppendOnlyData` object on the network.
        - Similarly to `MutableData`, we further sub-divide `AppendOnlyData` into two distinct sub-categories, `Sequenced` and `Unsequenced`.
          For `SequencedAppendOnlyData` the client MUST specify the next data index while appending.
          For `UnsequencedAppendOnlyData` the client does not have to pass the index.
        - We change the key-value container from a `BTreeMap` to a `Vec`. We normally avoided `HashMap` as by default the ordering is dependant on the hasher algorithm (hence these are called unordered maps) and we wanted all vaults to agree on the snapshot (with eventual consistency). The change to vector is because a vector is naturally chronological if we allow push-only operations. This allows us to index into the vector and use it in other places.
            - E.g., If the ownership change happened from `Owner-0` to `Owner-1` it might be of interest what the data was like when `Owner-0` owned it. So we can say that data index during the change was `n`. This means that in future after multiple owner changes we SHALL still have all the history preserved and can precisely show how the histories of data are related to the histories of metadata. So `[0-n]` would be all the data that was published when the owner was `Owner-0` and `[(n+1)..]` is the data when the owner was `Owner-1`, in our example. Similarly the permissions and the indices used there.
        - Appending new permission or ownership SHALL require the last index of the corresponding vector to be specified as part of the API call to prevent conflict, irrespective of whether the `AppendOnlyData` is `Sequenced` or `Unsequenced`.
- ABFT ordered consensus algorithm - `PARSEC` MUST be used for concurrency handling while catering to update requests.
- `AppendOnlyData` and `MutableData` use different XOR namespaces, but within sub-categories they are the same. I.e., if some user stored a `PublishedSequencedAppendOnlyData` object at a XOR address X and type tag Y, and another user wants to store `UnpublishedUnsequencedAppendOnlyData` at the same XOR address X and type tag Y, there will be a conflict and an error will be returned. The same rule applies for `SequencedMutableData` and `UnsequencedMutableData`: it SHALL NOT be possible to store them under the same XOR address and type tag. However, it SHALL be possible to store e.g. `UnpublishedSequencedAppendableData` and `SequencedMutableData` under the same XOR address and type tag.

```rust
/// In this type, data mutations must be explicitly sequenced by providing a version.
pub struct SequencedMutableData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    /// Key-Value semantics
    data: BTreeMap<Vec<u8>, Value>,
    /// Maps an application key to a list of allowed or forbidden actions
    permissions: BTreeMap<PublicKey, BTreeSet<Permission>>,
    /// Version should be increased for any changes to MutableData fields except for data
    version: u64,
    /// Contains a set of owners of this data. DataManagers enforce that a mutation request is
    /// coming from the MaidManager Authority of the Owner.
    owners: PublicKey,
}

/// Data mutations don't have to be explicitly sequenced and can go without a version.
/// All other mutations (permissions, owners, etc.) still MUST be sequenced.
pub struct UnsequencedMutableData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    /// Key-Value semantics
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    /// Maps an application key to a list of allowed or forbidden actions
    permissions: BTreeMap<PublicKey, BTreeSet<Permission>>,
    /// Version should be increased for any changes to MutableData fields except for data
    version: u64,
    /// Contains a set of owners of this data. DataManagers enforce that a mutation request is
    /// coming from the MaidManager Authority of the Owner.
    owners: PublicKey,
}

pub struct Value {
    /// Actual data.
    data: Vec<u8>,
    /// SHALL be incremented sequentially for any change to `data`.
    version: u64
}

/// This data is publicly available to any requester.
/// Any changes to ownership or permissions MUST be explicitly sequenced by providing a current index of `data` (`data.len()`).
/// Append operations MUST be explicitly sequenced by providing the index of the appended data entry.
pub struct PublishedSequencedAppendOnlyData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    /// History of appends - Key-Value Semantics.
    /// Keys can be used like index to query for versions
    data: Vec<(Vec<u8>, Vec<u8>)>,
    /// History of permission changes
    permissions: Vec<Permissions>,
    /// History of ownership changes
    owners: Vec<Owners>,
}

/// This data is publicly available to any requester.
/// Any changes to ownership or permissions MUST be explicitly sequenced by providing a current index of `data` (`data.len()`).
/// Append operations do not need to be explicitly sequenced.
pub struct PublishedUnsequencedAppendOnlyData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    /// History of appends - Key-Value Semantics.
    /// Keys can be used like index to query for versions
    data: Vec<(Vec<u8>, Vec<u8>)>,
    /// History of permission changes
    permissions: Vec<Permissions>,
    /// History of ownership changes
    owners: Vec<Owners>,
}

/// This data is available only to the owner(s) and explicitly permitted keys (other users, apps, etc.).
/// Any changes to ownership or permissions MUST be explicitly sequenced by providing a current index of `data` (`data.len()`).
/// Append operations MUST be explicitly sequenced by providing the index of the appended data entry.
pub struct UnpublishedSequencedAppendOnlyData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    /// History of appends - Key-Value Semantics.
    /// Keys can be used like index to query for versions
    data: Vec<(Vec<u8>, Vec<u8>)>,
    /// History of permission changes
    permissions: Vec<Permissions>,
    /// History of ownership changes
    owners: Vec<Owners>,
}

/// This data is available only to the owner(s) and explicitly permitted keys (other users, apps, etc.).
/// Any changes to ownership or permissions MUST be explicitly sequenced by providing a current index of `data` (`data.len()`).
/// Append operations do not need to be explicitly sequenced.
pub struct UnpublishedUnsequencedAppendOnlyData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    /// History of appends - Key-Value Semantics.
    /// Keys can be used like index to query for versions
    data: Vec<(Vec<u8>, Vec<u8>)>,
    /// History of permission changes
    permissions: Vec<Permissions>,
    /// History of ownership changes
    owners: Vec<Owners>,
}

struct Permissions {
    permissions: BTreeMap<User, BTreeSet<Permission>>,
    /// The current index of the data when this permission change happened
    data_index: u64,
    /// The current index of the owners when this permission change happened
    owner_entry_index: u64,
}

struct Owners {
    owners: PublicKey,
    /// The current index of the data when this ownership change happened
    data_index: u64,
    /// The current index of the permissions when this ownership change happened
    permission_entry_index: u64,
}

pub enum PublicKey {
    // To be defined by the implementation.
    // Can be a BLS public key, for example.
}
```



### Notes for the API for `AppendOnlyData`:
- The append of the data record MUST be done in one of the two ways:
    - Append given what the expected index of the appended data is going to be. When vaults receive this API they SHALL ensure that they push the entry _only_ if its index is going to be what the request claims after the append operation. This is useful in cases where the append operation only makes sense in an existing context.
    - Append without mentioning the expected index of the appended data. In this case vaults do no checking and SHALL simply push-back the data to the vector. This is useful if the user doesn't care about existing context (and hence possible conflicting updates) and is simply OK with concurrent appends in any order. This will usually be a much faster operation during concurrent appends as mutation requests are unlikely to be rejected due to index mismatch.

## Drawbacks

## Alternatives

- Instead of using separate data types use a single data type which internally behaves differently depending on the tag type. This however is prone to more maintenance and intellectual overhead (in terms of UX) than the static type safety of different data types.
- Instead of having separate data types for Sequenced and Unsequenced changes, have a boolean flag that would distinguish the data types. However, this is prone to the same disadvantages as the aforementioned option.
- Have completely separate namespaces for all 6 data types instead of using the same XOR namespace for sub-categories of `MutableData` and `AppendOnlyData`. With this approach, it SHALL be possible to store e.g. `UnpublishedSequencedAppendableData` and `PublishedSequencedAppendableData` at the same XOR address and type tag.

## Unresolved questions

0. It is not clear if the special `OwnerGet` RPC is worthwhile enough. The data is stored publicly by the vaults so is retrievable by them anyway. The Network doing extra checks and work to make certain data get-able only by the owner(s) (or keys authorised by the owner(s)) for something that is stored in public by the vaults and can be worked around with not much effort (e.g., published on clearnet etc.) seems like an artificial constraint. Also there isn't much incentive for the vaults to adhere to this behaviour. Can it in-fact, over a period of time, might become a norm for the vaults to just return data irrespective of the asker (which is the case now) for `GETs` and earn rewards for doing such work anyway ?
    - **Answer** from @dirvine: Gets SHALL go via MaidManagers. So `Client <-> MaidManagers <-> DataManagers`. The `MMs` SHALL ACK the `DMs`. If the `DM` vault was responding to a non-owner `GET` of an unpublished data then the group ACK of it from `MM -> DM` would prove it did so and SHALL be penalised accordingly. This should deincentivise vaults from responding to non-owner `GETs` of `MutableData`.


# Appendix: RPC specification

## Primitive atomic operations

We have 6 different data types in total, but most of them share the same set of primitive operations.

### AppendOnlyData

1. **Put**. Create a new `AppendOnlyData` instance on the network.
1. **Append entries**. Append one or more entries to an `AppendOnlyData` instance on the network. If the `AppendOnlyData` is sequenced, the request MUST include the index of a last entry. This operation can be performed only by user(s) having the corresponding permission.
1. **Get last entry**. Fetch the entry with the current index.
1. **Get entries range**. Fetch entries in the specified range (the ranges SHALL support relative indexing, e.g. "5 entries before the last entry").
1. **Get current indexes**. Fetch the current data index, permissions index, and owners index of this AppendOnlyData object.
1. **List permissions at index**. Fetch permissions at the provided index. The index in this operation refers to the index of the `permissions` list.
1. **List owners at index**. Fetch owners at the provided index. The index in this operation refers to the index of the `owners` list.
1. **List user permissions at index**. Get permissions for a provided user's/users' public key at the specified index. The index in this operation refers to the index of the `permissions` list.
1. **Append permissions**. Append a new list of permissions to the `AppendOnlyData` object. MUST include current indexes of data and permissions list. See the `Permissions` structure for reference. This operation can be performed only by the owner(s).
1. **Append owners**. Appends a new key to the owners list. MUST include current indexes of data and owners list. This operation can be performed only by the owner(s).
1. **Get shell at index**. Must include a version. If version is `Range::FromEnd(0)`, then the last shell version is going to be returned.
1. **Delete unpublished `AppendOnlyData`**. Completely removes an `AppendOnlyData` instance from the network. Does not apply to published `AppendOnlyData` which can not be unpublished or removed. This operation can be performed only by the owner(s).

### MutableData

1. **Get**. Get the entire Mutable Data object from the network.
1. **Put**. Put a new Mutable Data object on the network.
1. **Mutate entries**. Execute a list of actions on the Mutable Data object on the network. Reference the [RFC 47][rfc47] for more information.
1. **Get version**. Get the current shell version of Mutable Data on the network.
1. **Get shell**. Get the Mutable Data object without data it contains.
1. **List entries**. Get all entries of a Mutable Data object.
1. **List keys**. Get all keys of a Mutable Data object.
1. **List values**. Get all values of a Mutable Data Object.
1. **List permissions**. Get all permissions for a Mutable Data Object.
1. **List user permissions**. Get all permissions for a certain user (referenced by a public key).
1. **Set user permissions**. Set permissions for a certain user (referenced by a public key).
1. **Delete user permissions**. Deletes permissions for a certain user (referenced by a public key).
1. **Change ownership**. Sets a new owner or a set of owners (referenced by a public key) for a Mutable Data object on the network. This operation can be performed only by the owner(s).

## Detailed design

### Data types

```rust
struct AppendOnly {
    name: XorName,
    tag: u64,
    data: Vec<(Vec<u8>, Vec<u8>)>,
    permissions: Vec<Permissions>,
    owners: Vec<Owners>,
}

enum AppendOnlyKind {
    // Published, sequenced append-only data
    PubSeq,
    // Published, unsequenced append-only data
    PubUnseq,
    // Unpublished, sequenced append-only data
    UnpubSeq,
    // Unpublished, unsequenced append-only data
    UnpubUnseq,
}

enum MutableDataKind {
    // Unsequenced, unpublished Mutable Data
    Unsequenced,
    // Sequenced, unpublished Mutable Data
    Sequenced,
}

// Common public methods SHALL be implemented for this enum.
pub enum AppendOnlyData {
    // Published, sequenced append-only data
    PubSeq(AppendOnly),
    // Published, unsequenced append-only data
    PubUnseq(AppendOnly),
    // Unpublished, sequenced append-only data
    UnpubSeq(AppendOnly),
    // Unpublished, unsequenced append-only data
    UnpubUnseq(AppendOnly),
}

pub enum Range {
    // Use the specified index starting from 0.
    Index(u64),
    // Counting from the last index (`last_index - n`).
    FromEnd(u64),
}
```

### RPCs

The following document proposes to expand the protocol of communication of Clients with Vaults in the following way:

```rust
pub struct UnpublishedADataPermissionSet {
    read: Option<bool>,
    append: Option<bool>,
    manage_permissions: Option<bool>,
}

pub struct PublishedADataPermissionSet {
    append: Option<bool>,
    manage_permissions: Option<bool>,
}

pub struct MDataPermissionSet {
    read: Option<bool>,
    insert: Option<bool>,
    update: Option<bool>,
    delete: Option<bool>,
    manage_permissions: Option<bool>,
}

pub struct AppendOnlyDataRef {
    // Address of an AppendOnlyData object on the network.
    name: XorName,
    // Type tag.
    tag: u64,
}

pub struct MutableDataRef {
    // Address of a MutableData object on the network.
    name: XorName,
    // Type tag.
    tag: u64,
}

pub struct AppendOperation {
    // Address of an AppendOnlyData object on the network.
    address: AppendOnlyDataRef,
    // A list of entries to append.
    values: Vec<(Vec<u8>, Vec<u8>)>
}

pub enum User {
    Key(PublicKey),
    Anyone
}

pub enum Request {
    // ===== Append Only Data =====
    // Get a range of entries from an AppendOnlyData object on the network.
    GetADataRange {
        // Type of AppendOnlyData (published/unpublished, sequenced/unsequenced).
        kind: AppendOnlyKind,

        // Address of an AppendOnlyData object on the network.
        address: AppendOnlyDataRef,

        // Range of entries to fetch.
        //
        // For example, get 10 last entries:
        // range: (Range::FromEnd(10), Range::FromEnd(0))
        //
        // Get all entries:
        // range: (Range::Index(0), Range::FromEnd(0))
        //
        // Get first 5 entries:
        // range: (Range::Index(0), Range::Index(5))
        range: (Range, Range),
    },

    // Get current indexes: data, owners, permissions.
    GetADataIndexes {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
    },

    // Get an entry with the current index.
    GetADataLastEntry {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
    },

    // Get permissions at the provided index.
    GetADataPermissions {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
        permissions_index: Range,
    },

    // Get permissions for a specified user(s).
    GetADataUserPermissions {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
        permissions_index: Range,
        user: User,
    },

    // Get owners at the provided index.
    GetADataOwners {
        address: AppendOnlyDataRef,
        kind: AppendOnlyKind,
        owners_index: Range,
    },

    // Add a new `permissions` entry.
    // The `Permissions` struct instance MUST contain a valid index.
    AddADataPermissions {
        address: AppendOnlyDataRef,
        kind: AppendOnlyKind,
        // New permission set
        permissions: Permissions,
        permissions_index: u64,
    },

    // Add a new `owners` entry.
    // The `Owners` struct instance MUST contain a valid index.
    // Only the current owner(s) can perform this action.
    SetADataOwners {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
        owners: Owners,
        owners_index: u64,
    },

    // Get a specified key from AppendOnlyData.
    GetADataValue {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
        // Key to look up.
        key: Vec<u8>,
    },

    // Append operations
    AppendPublishedSeq {
        append: AppendOperation,
        index: u64,
    },
    AppendUnpublishedSeq {
        append: AppendOperation,
        index: u64,
    },
    AppendPublishedUnseq(AppendOperation),
    AppendUnpublishedUnseq(AppendOperation),

    // Put a new AppendOnlyData on the network.
    PutAData {
        // AppendOnlyData to be stored
        data: AppendOnlyData,
    },

    // Get `AppendOnlyData` shell at a certain point
    // in history (`index` refers to the list of data).
    GetADataShell {
        kind: AppendOnlyKind,
        address: AppendOnlyDataRef,
        data_index: Range,
    },

    // Delete an unpublished unsequenced `AppendOnlyData`.
    // Only the current owner(s) can perform this action.
    DeleteUnseqAData(AppendOnlyDataRef),
    // Delete an unpublished sequenced `AppendOnlyData`.
    // This operation MUST return an error if applied to published AppendOnlyData.
    // Only the current owner(s) can perform this action.
    DeleteSeqAData(AppendOnlyDataRef),

    // ===== Mutable Data =====
    // Fetches whole MutableData from the network.
    // Note: responses to this request are unlikely to accumulate during churn.
    GetUnseqMData {
        address: MutableDataRef,
    },
    GetSeqMData {
        address: MutableDataRef,
    /// Creates a new MutableData in the network.
    PutUnseqMData {
        // Mutable Data to be stored
        data: UnsequencedMutableData,
    },
    PutSeqMData {
        // Mutable Data to be stored
        data: SequencedMutableData,
    },
    // Fetches a latest version number.
    GetMDataVersion {
        address: MutableDataRef,
        kind: MutableDataKind,
    },
    // Fetches the shell (everything except the entries).
    GetUnseqMDataShell {
        address: MutableDataRef,
    },
    GetSeqMDataShell {
        address: MutableDataRef,
    },

    // -- Data Actions --
    // Fetches a list of entries (keys + values).
    // Note: responses to this request are unlikely to accumulate during churn.
    ListUnseqMDataEntries {
        address: MutableDataRef,
    },
    ListSeqMDataEntries {
        address: MutableDataRef,
    },
    // Fetches a list of keys in MutableData.
    // Note: responses to this request are unlikely to accumulate during churn.
    ListMDataKeys {
        address: MutableDataRef,
        kind: MutableDataKind,
    },
    // Fetches a list of values in MutableData.
    // Note: responses to this request are unlikely to accumulate during churn.
    ListUnseqMDataValues {
        address: MutableDataRef,
    },
    ListSeqMDataValues {
        address: MutableDataRef,
    },
    // Fetches a single value from MutableData
    GetUnseqMDataValue {
        // Network identifier of MutableData
        address: MutableDataRef,
        // Key of an entry to be fetched
        key: Vec<u8>,
    },
    GetSeqMDataValue {
        // Network identifier of MutableData
        address: MutableDataRef,
        // Key of an entry to be fetched
        key: Vec<u8>,
    },
    // Updates MutableData entries in bulk.
    MutateMDataEntries {
        // Network identifier of MutableData
        address: MutableDataRef,
        // A list of mutations (inserts, updates, or deletes) to be performed
        // on MutableData in bulk.
        actions: BTreeMap<Vec<u8>, EntryAction>,
    },

    // -- Permission Actions --
    // Fetches a complete list of permissions.
    ListMDataPermissions {
        address: MutableDataRef,
        kind: MutableDataKind,
    },
    // Fetches a list of permissions for a particular User.
    ListMDataUserPermissions {
        address: MutableDataRef,
        kind: MutableDataKind,
        user: PublicKey,
    },
    // Updates or inserts a list of permissions for a particular User in the given MutableData.
    SetMDataUserPermissions {
        // Network identifier of MutableData
        address: MutableDataRef,
        // Kind of Mutable Data
        kind: MutableDataKind,
        // A user identifier used to set permissions
        user: PublicKey,
        // Permissions to be set for a user
        permissions: MDataPermissionSet,
        // Incremented version of MutableData
        version: u64,
    },
    // Deletes a list of permissions for a particular User in the given MutableData.
    DelMDataUserPermissions {
        /// Network identifier of MutableData
        address: MutableDataRef,
        // Kind of Mutable Data
        kind: MutableDataKind,
        // A user identifier used to delete permissions
        user: PublicKey,
        // Incremented version of MutableData
        version: u64,
    },
    // -- Ownership Actions --
    // Changes an owner of the given MutableData. Only the current owner(s) can perform this action.
    ChangeMDataOwner {
        // Network identifier of MutableData
        address: MutableDataRef,
        // Kind of Mutable Data
        kind: MutableDataKind,
        // A list of new owners
        new_owners: PublicKey,
        // Incremented version of MutableData
        version: u64,
    },
    // Delete a Mutable Data object from the network.
    // Only the current owner(s) can perform this action.
    DeleteMData {
        // Network identifier of MutableData
        address: MutableDataRef,
        // Kind of Mutable Data
        kind: MutableDataKind,
    }
}

pub struct Indexes {
    data_index: u64,
    owners_index: u64,
    permissions_index: u64,
}

pub enum Response {
    // === AppendOnlyData ===
    GetADataRange(Result<Vec<(Vec<u8>, Vec<u8>)>, ClientError>),
    GetADataIndexes(Result<Indexes, ClientError>),
    GetADataLastEntry(Result<(Vec<u8>, Vec<u8>)>, ClientError),
    GetADataPermissions(Result<Permissions, ClientError>),
    GetADataUserPermissions(Result<BTreeSet<Permission>, ClientError>),
    GetADataOwners(Result<Owners, ClientError>),
    GetADataValue(Result<Vec<u8>, ClientError>),
    // Returns a last data index.
    AppendPublishedUnsequenced(Result<u64, ClientError>),
    // Returns a last data index.
    AppendUnpublishedUnsequenced(Result<u64, ClientError>),
    GetADataShell(Result<AppendOnlyData, ClientError>),

    // ===== Mutable Data =====
    GetUnseqMData(Result<UnsequencedMutableData, ClientError>),
    GetSeqMData(Result<SequencedMutableData, ClientError>),
    GetMDataVersion(Result<u64, ClientError>),
    GetUnseqMDataShell(Result<UnsequencedMutableData, ClientError>),
    GetSeqMDataShell(Result<SequencedMutableData, ClientError>),
    ListUnseqMDataEntries(Result<BTreeMap<Vec<u8>, Vec<u8>>, ClientError>),
    ListSeqMDataEntries(Result<BTreeMap<Vec<u8>, Value>, ClientError>),
    ListMDataKeys(Result<BTreeSet<Vec<u8>>, ClientError>),
    ListUnseqMDataValues(Result<Vec<Vec<u8>>, ClientError>),
    ListSeqMDataValues(Result<Vec<Value>, ClientError>),
    GetUnseqMDataValue(Result<Vec<u8>, ClientError>),
    GetSeqMDataValue(Result<Value, ClientError>),
    ListMDataPermissions(Result<BTreeMap<PublicKey, BTreeSet<MDataPermission>>>),
    ListMDataUserPermissions(Result<BTreeSet<MDataPermission>>),
    // Common to all mutation operations
    Mutation(Result<(), ClientError>),
}
```

Please notice that the `Request` enum variants don't include the `msg_id` field anymore. Because every single variant need it, it would make sense to include it externally as a part of a request (e.g. a tuple `(MsgId, Request)` or a struct `struct RequestMsg { msg_id: MsgId, request: Request }`). Alternatively, we could leave it as it is.

### Note about SAFE Client Libs API

To simplify the burden of working with multiple different data types, it is RECOMMENDED to have a unified API to work with `MutableData` and with `AppendOnlyData`. We SHOULD be able to create these data types using the builder pattern. E.g.:

```rust
AppendOnly::unseq()
    .published()
    .owned_by(my_public_key)
    .with_permissions(perm_set)
    .build();
```

Similarly for `MutableData`:

```rust
MutableData::seq()
    .owned_by(my_public_key)
    .build();
```


[rfc47]: https://github.com/maidsafe/rfcs/blob/master/text/0047-mutable-data/0047-mutable-data.md
