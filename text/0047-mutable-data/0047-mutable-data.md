# MutableData

- Status: Implemented
- Type: feature
- Related components: `safe_core`, `safe_launcher`, `safe_vault`, `routing`
- Start Date: 09-November-2016
- Discussion: https://forum.safedev.org/t/rfc-47-mutabledata/295

## Summary
Combining `StructuredData` and `AppendableData` into a new single data type, `MutableData`, similar to HashMap.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
Existing data types don't support granular access control which is required for mobile platforms where access to the network through an intermediary like SAFE Launcher is complicated: each app would have to receive full permissions to modify and fetch data on a user's behalf and private keys to sign it. This compromises security and allows malicious apps to get full access to the user's data.

To mitigate that, `MutableData` provides fine-grained access control and more narrow-scoped data operations. This data type is an evolved version of `AppendableData`. It combines `StructuredData` and `AppendableData` features and improves the network efficiency, saves bandwidth, and provides a standardised approach for apps to fetch and mutate data in the network.

## Detailed design
The proposed data type defines discrete actions on data that doesn't require a user to replace or fetch the entire structure contents. This is enabled by functionality similar to this of the HashMap data structure or a key-value database such as [Redis](http://redis.io).

The structure itself is defined as following:

```rust
pub struct MutableData {
    /// Network address
    name: XorName,
    /// Type tag
    tag: u64,
    // ---- owner and vault access only ----
    /// Maps an arbitrary key to a (version, data) tuple value
    data: BTreeMap<Vec<u8>, Value>,
    /// Maps an application key to a list of allowed or forbidden actions
    permissions: BTreeMap<User, BTreeSet<Permission>>,
    /// Version should be increased for every change in MutableData fields
    /// except for data
    version: u64,
    /// Contains a set of owners which are allowed to mutate permissions.
    /// Currently limited to one owner to disallow multisig.
    owners: BTreeSet<sign::PublicKey>,
}
```
```rust
/// A value in MutableData
pub struct Value {
    content: Vec<u8>,
    entry_version: u64,
}
```
### Routing implementation
This RFC defines new entries in `Response` and `Request` enums on the routing side.

#### Data requests
These operations allow to fetch and mutate parts of data without the necessity to have complete information.
Actions performed on data are defined as follows:
```rust
pub enum EntryAction {
    Update(Value),
    Ins(Value),
    Del(u64),
}

pub enum Request {
    …
    /// Creates a new MutableData in the network
    PutMData {
        data: MutableData,
        msg_id: MsgId,
        requester: sign::PublicKey,
    },
    /// Fetches a latest version number of the provided MutableData
    GetMDataVersion {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    /// Fetches a list of entries (keys + values) of the provided MutableData
    ListMDataEntries {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    /// Fetches a list of keys of the provided MutableData
    ListMDataKeys {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    /// Fetches a list of values of the provided MutableData
    ListMDataValues {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    /// Fetches a single value from the provided MutableData by the given key
    GetMDataValue {
        name: XorName,
        tag: u64,
        key: Vec<u8>,
        msg_id: MsgId,
    },
    /// Updates MutableData entries in bulk
    MutateMDataEntries {
        name: XorName,
        tag: u64,
        /// actions is a map from Key to EntryAction (insert, delete, update)
        actions: BTreeMap<Vec<u8>, EntryAction>,
        msg_id: MsgId,
        requester: sign::PublicKey,
    },
    …
}
```
#### Client interface
Each request has a corresponding function in the routing Client to perform the action; the functions should be defined as following:
```rust
/// Fetches a latest version number of the provided MutableData
pub fn get_mutable_data_version(&self,
                                dst: Authority,
                                name: XorName,
                                tag: u64,
                                msg_id: MsgId);
```
```rust
/// Fetches a list of entries (keys + values) of the provided MutableData
pub fn list_mutable_data_entries(&self,
                                dst: Authority,
                                name: XorName,
                                tag: u64,
                                msg_id: MsgId);
```
```rust
/// Fetches a list of keys of the provided MutableData
pub fn list_mutable_data_keys(&self,
                             dst: Authority,
                             name: XorName,
                             tag: u64,
                             msg_id: MsgId);
```
```rust
/// Fetches a list of values of the provided MutableData
pub fn list_mutable_data_values(&self,
                               dst: Authority,
                               name: XorName,
                               tag: u64,
                               msg_id: MsgId);
```
```rust
/// Fetches a single value from the provided MutableData by the given key
pub fn get_mutable_data_value(&self,
                              dst: Authority,
                              name: XorName,
                              tag: u64,
                              key: Vec<u8>,
                              msg_id: MsgId);
```

The above-mentioned requests must be handled by the `DataManger` persona in vaults, skipping authentication and authorisation checks in `MaidManager`, as `get` and `list` requests are always allowed.

The following operations must be handled by `MaidManager`:
```rust
/// Creates a new MutableData in the network
pub fn put_mutable_data(&self,
                        dst: Authority,
                        data: MutableData,
                        msg_id: MsgId,
                        requester: sign::PublicKey);
```
```rust
/// Updates MutableData entries in bulk
pub fn mutate_mutable_data_entries(&self,
                                   name: XorName,
                                   tag: u64,
                                   actions: BTreeMap<Vec<u8>, EntryAction>,
                                   msg_id: MsgId,
                                   requester: sign::PublicKey);
```

Client can also define convenience functions that will create a `MutateMDataEntries` request for a user and send it:
```rust
/// Updates a single entry in the provided MutableData by the given key
pub fn update_mutable_data_value(&self,
                                 dst: Authority,
                                 name: XorName,
                                 tag: u64,
                                 key: Vec<u8>,
                                 value: Value,
                                 msg_id: MsgId,
                                 requester: sign::PublicKey);
```
```rust
/// Inserts a new entry (key-value pair) to the provided MutableData
pub fn insert_mutable_data_entry(&self,
                                 dst: Authority,
                                 name: XorName,
                                 tag: u64,
                                 key: Vec<u8>,
                                 value: Value,
                                 msg_id: MsgId,
                                 requester: sign::PublicKey);
```
```rust
/// Deletes a single entry from the provided MutableData by the given key
pub fn delete_mutable_data_entry(&self,
                                 dst: Authority,
                                 name: XorName,
                                 tag: u64,
                                 key: Vec<u8>,
                                 entry_version: u64,
                                 msg_id: MsgId,
                                 requester: sign::PublicKey);
```

#### Responses
```rust
/// Error part of the Result type is the error reason
pub type ResonseResult<T> = Result<T, Vec<u8>>;

pub enum Response {
    …
    // MutableData actions
    PutMData(ResponseResult<()>, MsgId),
    GetMDataVersion(ResponseResult<u64>, MsgId),

    // Data actions
    ListMDataEntries(ResponseResult<BTreeMap<Vec<u8>, Value>>, MsgId),
    ListMDataKeys(ResponseResult<BTreeSet<Vec<u8>>>, MsgId),
    ListMDataValues(ResponseResult<Vec<Value>>, MsgId),
    GetMDataValue(ResponseResult<Value>, MsgId),
    MutateMDataEntries(ResponseResult<()>, MsgId),
    …
}
```

### Permissions
`MutableData` allows to control permissions by providing an access control list within the `permissions` field defined as following:
```rust
pub struct MutableData {
    …
    permissions: BTreeMap<User, BTreeSet<Permission>>,
    …
}

pub enum User {
    Anyone,
    Key(sign::PublicKey),
}

pub enum Permission {
    Allow(User),
    Deny(User),
}

pub enum Action {
    Insert,
    Update,
    Delete,
    ManagePermission,
}
```

`User` defines an access subject which can be a public key of an app (`User::Key(K1)`) or a wildcard value for any public key (`User::Anyone`). `User::Key` must override all permissions defined for `User::Anyone`.

For example, let's assume a scenario where a user wants to allow adding comments to a blog post for anyone but a spammer with a key K1. In this case, the following permission set should be used:

```rust
permissions: {
    User::Anyone → [
        Permission::Allow(Action::Insert),
        Permission::Deny(Action::Update),
    ],
    User::Key(K1) → [
        Permission::Deny(Action::Insert),
    ],
}
```

Notice that as `User::Anyone` allows everyone to update comments and `User::Key(K1)` forbids only insertion of new comments, it will still allow an owner of the key `K1` to update his comments.

This model provides the same functionality as blacklists and whitelists in `AppendableData`. Whitelist can be defined as a permission list with a specific set of keys without a `User::Anyone` entry.

### Requests
Permissions can be modified granularly by issuing actions similar to those used to update data.
Only owners and apps having the `ManagePermission` permission must be allowed to issue such requests.

These requests defined as following on the routing side:
```rust
pub enum Request {
    ListMDataPermissions {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    ListMDataUserPermissions {
        name: XorName,
        tag: u64,
        user: User,
        msg_id: MsgId,
    },
    SetMDataUserPermissions {
        name: XorName,
        tag: u64,
        user: User,
        permissions: BTreeSet<Permission>,
        version: u64,
        msg_id: MsgId,
        requester: sign::PublicKey,
    },
    DelMDataUserPermissions {
        name: XorName,
        tag: u64,
        user: User,
        version: u64,
        msg_id: MsgId,
        requester: sign::PublicKey,
    },
}
```
#### Client interface
Corresponding functions should be defined in Client:

```rust
pub fn list_mutable_data_permissions(&self,
                                    dst: Authority,
                                    name: XorName,
                                    tag: u64,
                                    msg_id: MsgId);
```
```rust
pub fn list_mutable_data_user_permissions(&self,
                                    dst: Authority,
                                    name: XorName,
                                    tag: u64,
                                    user: User,
                                    msg_id: MsgId);
```
```rust
pub fn set_mutable_data_user_permissions(&self,
                                    dst: Authority,
                                    name: XorName,
                                    tag: u64,
                                    user: User,
                                    permissions: BTreeSet<Permission>,
                                    version: u64,
                                    msg_id: MsgId,
                                    requester: sign::PublicKey);
```
```rust
pub fn del_mutable_data_user_permissions(&self,
                                         dst: Authority,
                                         name: XorName,
                                         tag: u64,
                                         user: User,
                                         version: u64,
                                         msg_id: MsgId,
                                         requester: sign::PublicKey);
```

#### Responses
```rust
pub enum Response {
    …
    ListMDataPermissions(ResponseResult<BTreeMap<User, BTreeSet<Permission>>, MsgId),
    ListMDataUserPermissions(ResponseResults<BTreeSet<Permission>>, MsgId),
    SetMDataUserPermissions(ResponseResult<()>, MsgId),
    DelMDataUserPermissions(ResponseResult<()>, MsgId),
    …
}
```

### Ownership transfer
A `MutableData` owner can transfer ownership by issuing a special request:
```rust
pub enum Request {
    …
    /// Requests an owner change
    ChangeMDataOwner {
        /// MutableData identifier
        name: XorName,
        /// MutableData type tag
        tag: u64,
        /// New owners
        new_owners: BTreeSet<sign::PublicKey>,
        /// Must be equal to (last known `MutableData` version + 1)
        version: u64,
        msg_id: MsgId
    },
    …
}
```

Clients should define the following function to send the ownership transfer RPC requests:
```rust
pub fn send_ownership_transfer(&self,
                               dst: Authority,
                               name: XorName,
                               tag: u64,
                               new_owners: BTreeSet<sign::PublicKey>,
                               version: u64,
                               message_id: MessageId);
```

Response format:
```rust
pub enum Response {
    …
    ChangeMDataOwner(ResponseResult<()>, MsgId),
    …
}
```

### Vaults implementation
The actual work is performed by vaults if the signature is valid and the requester has sufficient privileges.

#### Versions
Versions are necessary because of data churn.

#### Refresh messages
When a node joins the group it receives refresh messages in the following order:

1. Refresh message for `MutableData` without entries. It must contain its version, its name, and a hash of `MutableData` excluding the `data` field:

    ```rust
    struct RefreshData(IdAndVersion, u64);
    ```

    As soon as this message is accumulated, data holders (nodes that hold the actual `MutableData`) must be contacted one-by-one. That is, if the first data holder doesn't contain the requested data the second holder must be contacted.

    After the actual `MutableData` is retrieved it must be written to the chunk store.

2. Refresh messages for map entries. They must contain a hash of an individual key-value pair (key + entry value `Vec<u8>` + entry version), an entry version, and a network identifier of the parent `MutableData`. This refresh message must be issued only after the 1<sup>st</sup> refresh message is sent.

    ```rust
    struct RefreshMutableData {
        name: DataIdentifier,
        entry_version: u64,
        hash: u64
    }
    ```

    As soon as these messages are accumulated, data holders must be contacted one-by-one to get the entries by their hashes. The retrieved data must be written to the chunk store and added to the `MutableData`.

    In the case if a vault can't find `MutableData` by the provided identifier in its chunk store, an empty `MutableData` must be created and stored with that identifier:

    ```rust
    MutableData {
        tag: 0,
        data: BTreeMap::new(), // must contain the retrieved entry (key + value)
        permissions: BTreeMap::new(),
        version: 0,
        owners: BTreeSet::new(),
    }
    ```

#### Pending writes
Pending writes must support granular changes in data or permissions. Ownership transfer and encrypt key changes also must be represented by separate pending writes.

### Limits
The `MutableData` data type imposes the following limits:

* Maximum size for a serialised `MutableData` structure must be 1MiB;
* Maximum entries per `MutableData` must be 100;
* Not more than 5 simultaneous mutation requests are allowed for a (MutableData data identifier + type tag) pair;
* Only 1 mutation request is allowed for a single key.

#### Caveats
It is possible for `MutableData` to get over these limits because of eventual consistency handling by vaults: a size of a serialised `MutableData` can exceed 1MiB limit and a number of entries can exceed 100. Consider this example case:

1. Vaults **A** and **B** both have a mutable data **MD** with 99 entries and serialised size of 0.1MiB.
2. Both vaults get 2 simultaneous requests to store entries **E1** and **E2** with a total size of 0.9MiB each.
3. Vault **A** accumulates the entry **E1** before **E2** and vault **B** accumulates the entry **E2** before **E1**.
4. Now vaults **A** and **B** have an inconsistent set of entries. Both vaults **A** and **B** will eventually accumulate the other entry and write it into the chunk store which will make them consistent.
5. At the same time **MD** in both vaults will be conditioned to have 101 entries and serialised size of 1.9MiB.
