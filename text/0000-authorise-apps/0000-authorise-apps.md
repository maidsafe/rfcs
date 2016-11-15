# Authorise apps on behalf of the owner to mutate data
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_vault`, `routing`, `safe_launcher`
- Start Date: 10-November-2016
- Discussion: https://placeholder.com

## Summary
Detail how apps route their mutation requests to Data-Managers and how revocation works.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- When we talk about an owner's Maid-Managers we refer to the address `sha256::hash(owner's-MAID-sign-PublicKey)`.

## Motivation
- If the requests made by the permitted apps need to be charged these must all go through the Maid-Managers and they should know about the owner that an app is associated with so that they can charge the correct account.

## Detailed design
We require all mutations to go via the owner's Maid-Managers. Maid-Managers will keep a list of `sign::PublicKey` which is the representation of the apps the owner has allowed to mutate the network on their behalf. Maid-Managers will then forward the request to the appropriate Data-Managers. The type `MutableData` thus does not need to store any signatures and Data-Managers will use the group authority to trust the requester of the mutation. Then they will use the `permissions` field to allow or disallow the mutation itself.

New routing messages for request and response will be required to deal with `MutableData` rpcs as listed in the corresponding [RFC here](). These shall be the following:
```rust
pub enum EntryAction {
    Update(Value),
    Ins(Value),
    Del(u64),
}

pub enum Request {
    Refresh(Vec<u8>, MsgId),
    GetAccountInfo(MsgId),

    /// --- ImmutableData ---
    /// ==========================
    PutIData { data: ImmutableData, msg_id: MsgId },
    GetIData { name: XorName, msg_id: MsgId },

    /// --- MutableData ---
    /// ==========================
    PutMData {
        data: MutableData,
        msg_id: MsgId,
        requester: sign::PublicKey,
    },
    GetMDataVersion {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },

    /// Data Actions
    ListMDataEntries {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    ListMDataKeys {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    ListMDataValues {
        name: XorName,
        tag: u64,
        msg_id: MsgId,
    },
    GetMDataValue {
        name: XorName,
        tag: u64,
        key: Key,
        msg_id: MsgId,
    },
    MutateMDataEntries {
        name: XorName,
        tag: u64,
        actions: BTreeMap<Vec<u8>, EntryAction>,
        msg_id: MsgId,
        requester: sign::PublicKey,
    },

    /// Permission Actions
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

    /// Ownership Actions
    ChangeMDataOwner {
        name: XorName,
        tag: u64,
        new_owners: BTreeSet<sign::PublicKey>,
        version: u64,
        msg_id: MsgId,
    },

    /// --- Client (Owner) to MM ---
    /// ==========================
    ListAuthKeysAndVersion(MsgId),
    InsAuthKey {
        key: sign::PublicKey,
        version: u64,
        msg_id: MsgId,
    },
    DelAuthKey {
        key: sign::PublicKey,
        version: u64,
        msg_id: MsgId,
    },
}

pub enum Response {
    GetAccountInfoFailure { reason: Vec<u8>, msg_id: MessageId },
    GetAccountInfoSuccess {
        data_stored: u64,
        space_available: u64,
        msg_id: MessageId,
    },

    /// --- ImmutableData ---
    /// ==========================
    PutIData {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },
    GetIData {
        res: Result<ImmutableData, Vec<u8>>,
        msg_id: MsgId,
    },

    /// --- MutableData ---
    /// ==========================
    PutMData {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },

    GetMDataVersion {
        res: Result<u64, Vec<u8>>,
        msg_id: MsgId,
    },

    /// Data Actions
    ListMDataEntries {
        res: Result<BTreeMap<Vec<u8>, Value>, Vec<u8>>,
        msg_id: MsgId,
    },
    ListMDataKeys {
        res: Result<BTreeSet<Vec<u8>>, Vec<u8>>,
        msg_id: MsgId,
    },
    ListMDataValues {
        res: Result<Vec<Value>, Vec<u8>>,
        msg_id: MsgId,
    },
    GetMDataValue {
        res: Result<Value, Vec<u8>>,
        msg_id: MsgId,
    },
    MutateMDataEntries {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },

    /// Permission Actions
    ListMDataPermissions {
        res: Result<BTreeMap<User, BTreeSet<Permission>>, Vec<u8>>,
        msg_id: MsgId,
    },
    ListMDataUserPermissions {
        res: Result<BTreeSet<Permission>, Vec<u8>>,
        msg_id: MsgId,
    },
    SetMDataUserPermissions {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },
    DelMDataUserPermissions {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },

    /// Ownership Actions
    ChangeMDataOwner {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },

    /// --- Client (Owner) to MM ---
    /// ==========================
    ListAuthKeysAndVersion {
        res: Result<BTreeSet<sign::PublicKey>, Vec<u8>>,
        msg_id: MsgId,
    },
    InsAuthKey {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },
    DelAuthKey {
        res: Result<(), Vec<u8>>,
        msg_id: MsgId,
    },
}
```

Along with the requests and responses for `MutableData` the above also lists them for `ImmutableData`. `List` in the names of variants above indicates that requests/responses include a collection while `Get` implies a single value.

As we can see the data types lack the signature field. Mutation of `MutableData` has many rpc's dedicated for it: `put_mdata`, `set_mdata_user_permissions` etc., the details of which can be found in the rfc that details the type.

### The rules for `Authenticator` are:

- Generate a sign key-pair for the app: `SKP = (sign_pk, sign_sk) = sign::gen_keypair()`
- Ask Maid-Manager to map `sign_pk` against its (the owner's) account. The rpc for this is mentioned in this [RFC]().
The actual set of operations is detailed in the `Authenticator` [RFC]().

### The rules at Maid-Managers are:

- When handling `PUT` for `MutableData` Maid-Managers shall enforce that the incoming message's destination authority name equals the hash of the incoming message's `MutableData::owners`.
- For all mutation requests, the sender app, recognised by its `sign::PublicKey`, must be listed with the Maid-Managers. The owner's account will be charged if required and the request will be forwarded to the appropriate Data-Managers. While forwarding the request to the Data-Managers, the Maid-Managers will assert that the requester is  the the sign key of the app that made the request (the client source authority).
- Ownership transfers, are only allowed by the owner.

This is how the Maid-Manager account info will look like:
```rust
pub struct Account {
    data_stored: u64,
    space_available: u64,
    auth_keys: BTreeSet<sign::PublicKey>,
    version: u64,
}
```
The RPCs for managing the authenticated keys (i.e. keys that can be used to mutate on behalf of the owner-keys) shall be:
```rust
fn ins_auth_key(&self,
                dst: Authority, // Maid-Manager
                key: sign::PublicKey,
                version: u64,
                message_id: MessageId);

// ... and similar functions which correspond to the routing Request enum.
```
The Maid-Managers shall enforce that auth-key management is only allowed by the owner of the account (e.g. `Authenticator`). To know the current `version` the `Account` is at, the `get_auth_keys_and_version` rpc shall be used.

### The rules at Data-Managers are:

- All mutations need to be come via Maid-Managers.
- Only Get requests are permitted from clients directly.
- For ownership transfers enforce the incoming message source authority is Maid-Manager of data's owner.
- For mutations of `MutableData` it will use the `sign::PublicKey` obtained from routing which will extract it from `requester` field of `Request` message to verify the permissions associated with it.

### Transfer of ownership
Ownership transfers are a special case. The apps are not allowed to do this operation. They will need to ask the `Authenticator` to do this. Maid-Managers shall enforce that `change_mdata_owner` only comes from the owner. Data-Managers shall enforce that current owner of the data matches the incoming Maid-Manager group.

### Revocation
Once the owner decides to revoke permission of an app to mutate network on their behalf, they will go to the Maid-Managers and delete `sign_pk` for the app. Hence the app can no longer perform any mutating operations on behalf of the owner now as the owner's Maid-Managers will reject it not finding it in the list.
