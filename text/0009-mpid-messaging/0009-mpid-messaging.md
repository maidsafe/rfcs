# MPID Messaging System

- Status: proposed
- Type: New Feature
- Related Components: [safe_vault](https://github.com/maidsafe/safe_vault), [safe_client](https://github.com/maidsafe/safe_client), [mpid_messaging](https://github.com/maidsafe/mpid_messaging)
- Start Date: 22-09-2015
- RFC PR: #43
- Issue number: Agreed - #50
- Discussion: https://github.com/maidsafe/rfcs/issues/96
- Supersedes:
- Superseded by:

## Summary

This RFC outlines the system components and design for general communications infrastructure and security on the SAFE Network.

## Motivation

### Rationale

A messaging system on the SAFE Network will obviously be useful. Over and above this, the proposed messaging system is secure and private without the possibility of snooping or tracking.

A large abuse of modern day digital communications is the ability of spammers and less than ethical marketing companies to flood the Internet with levels of unsolicited email to a level of circa 90%. This is a waste of bandwidth and a significant nuisance.

### Supported Use-Cases

The system will support secure, private, bi-directional communications between pairs of Clients.

### Expected Outcome

The provision of a secure messaging system which will eradicate unwanted and intrusive communications.

## Detailed design

### Overview

A fundamental principle is that the cost of messaging is with the sender, primarily to deter unsolicited messages. To achieve this, the sender will maintain messages in a network [outbox][3] until they are retrieved by the recipient. The recipient will have a network [inbox][4] comprising a list of metadata relating to messages which are trying to be delivered to it.

If the message is unwanted, the recipient simply does not retrieve the message.  The sender will thus quickly fill their own outbox with undelivered mail and be forced to clean this up themselves before being able to send further messages.

This paradigm shift will mean that the obligation to unsubscribe from mailing lists, etc. is now with the owner of these lists. If people are not picking up messages, it is because they do not want them. So the sender has to do a better job. It is assumed this shift of responsibilities will lead to a better-managed bandwidth solution and considerably less stress on the network and the users of the network.

### Implementation Details

The two relevant structs (other than the MPID itself which is a [standard Client key][0]) are the [`MpidHeader`][1] and the [`MpidMessage`][2].

Broadly speaking, the `MpidHeader` contains metadata and the `MpidMessage` contains a signed header and message. We also want to define some upper limits which will be described later.

#### Constants

```rust
pub const MPID_MESSAGE: u64 = 51000;
pub const MAX_HEADER_METADATA_SIZE: usize = 128;  // bytes
pub const MAX_BODY_SIZE: usize = 102400 - 512 - MAX_HEADER_METADATA_SIZE;
pub const MAX_INBOX_SIZE: usize = 1 << 27;       // bytes, i.e. 128 MiB
pub const MAX_OUTBOX_SIZE: usize = 1 << 27;      // bytes, i.e. 128 MiB
```

#### `MpidHeader`

```rust
pub struct MpidHeader {
    sender_name: XorName,
    guid: [u8; 16],
    metadata: Vec<u8>,
    signature: ::sodiumoxide::crypto::sign::Signature,
}
```

The `sender` field is hopefully self-explanatory. The `guid` allows the message to be uniquely identified, both by receivers and by the Manager Vaults which need to hold the messages in a map-like structure. The signature contains the cryptographic evidence over the values of `sender_name`, `guid` and `metadata` (in this order) signed by the secret key of the sender.

The `metadata` field allows passing arbitrary user/app data. It must not exceed `MAX_HEADER_METADATA_SIZE` bytes.

#### `MpidMessage`

```rust
pub struct MpidMessage {
    header: MpidHeader,
    recipient: XorName,
    body: Vec<u8>,
    recipient_and_body_signature: ::sodiumoxide::crypto::sign::Signature,
}
```
Each `MpidMessage` instance only targets one recipient. For multiple recipients, multiple `MpidMessage`s need to be created in the [Outbox][3] (see below). This is to ensure spammers will run out of limited resources quickly.

### Outbox

This is a simple data structure for now and will be a hash map of serialised and encrypted `MpidMessage`s. There will be one such map per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

This can be implemented as a `Vec<MpidMessage>`.

### Inbox

Again this will be one per MPID (owner), held on the `MpidManager`s, and synchronised by them at churn events.

This can be implemented as a `Vec<(sender_name: XorName, sender_public_key: ::sodiumoxide::crypto::sign::PublicKey, mpid_header: MpidHeader)>` or having the headers from the same sender grouped: `Vec<(sender_name: XorName, sender_public_key: ::sodiumoxide::crypto::sign::PublicKey, headers: Vec<mpid_header: MpidHeader>)>` (however this may incur a performance slow down when looking up for a particular mpid_header).

### Messaging Format Among Nodes

Messages between Clients and MpidManagers will utilise [`::routing::plain_data::PlainData`][5], for example:
```rust
let pd = PlainData {
    name: mpid_message_name(mpid_message),
    data: serialise(MpidMessageWrapper::PutMessage(mpid_message)),
}
```

When handling churn, MpidManagers will synchronise their outboxes by sending each included MpidMessage as a single network message. However, since MpidHeaders are significantly smaller, inboxes will be sent as a message containing a vector of all contained headers.

### Message Flow

![Flowchart showing MPID Message flow through the SAFE Network](MPID%20Message%20Flow.png)

### MPID Client

The MPID Client shall provide the following key functionalities :

1. Send Message (`Put` from sender)
1. Accept Message header (`Push` from `MpidManager`s to recipient)
1. Retrieve Full Message (`Get` from receiver)
1. Query own inbox to get list of all remaining `MpidHeader`s
1. Query own inbox for a vector of specific `MpidHeader`s to see whether they remain in the outbox or not.
1. Remove unwanted `MpidHeader` (`Delete` from recipient)
1. Query own outbox to get list of all remaining `MpidMessage`s
1. Remove sent Message (`Delete` from sender)

If the `Push` model is used, an MPID Client is expected to have its own routing object (not shared with the MAID Client). In this way it can directly connect to its own `MpidManager`s (or the connected `ClientManager` will register itself as the proxy to the corresponding `MpidManager`s), allowing them to know its online status and hence they can push message headers to it as and when they arrive.

Such a separate routing object (or the registering procedure) is not required if the `Pull` model is employed. This is where the MPID Client periodically polls its network inbox for new headers. It may also have the benefit of saving the battery life on mobile devices, as the client app doesn't need to keep MPID Client running all the time.

### Planned Work

1. MPID-Messaging
    1. Definition of `MpidMessage`.
    1. Definition of `MpidHeader`.
    1. Definition of `Constant`s.
    1. Structure of `PlainData::value` for messaging.

1. Vault
    1. Definition of `Outbox`.
    1. Definition of `Inbox`.
    1. Sending message flow.
    1. Retrieving message flow.
    1. Deleting message flow.
    1. Churn handling and refreshing for account_transfer (Inbox and Outbox).
    1. MPID Client registering (when GetAllHeader request received).
    1. `Authority::MpidManager`.

1. Client
    1. Put `MpidMessage`.
    1. Get all `MpidHeader`s (pull).
    1. Accept all/single `MpidHeader` (push).
    1. Get `MpidMessage`. This shall also include the work of removing corresponding `MpidHeader`s.
    1. Delete `MpidMessage`.
    1. Delete `MpidHeader`.


## Drawbacks

None identified, other than increased complexity of Vault and Client codebase.

## Alternatives

No other in-house alternatives have been documented as yet.

If we were to _not_ implement some form of secure messaging system, a third party would be likely to implement a similar system using the existing interface to the SAFE Network. This would be unlikely to be as efficient as the system proposed here, since this will be "baked into" the Vault protocols.

We also have identified a need for some form of secure messaging in order to implement safecoin, so failure to implement this RFC would impact on that too.

## Unresolved Questions

None.

## Future Work

- This RFC doesn't address how Clients get to "know about" each other. Future work will include details of how Clients can exchange MPIDs in order to build an address book of peers.

- It might be required to provide Vault-to-Vault, Client-to-Vault or Vault-to-Client communications in the future. Potential use cases for this are:

    1. Vault-to-Client notification of a successful safecoin mining attempt
    1. Client-to-Vault request to take ownership of the Vault
    1. Vault-to-Client notification of low resources on the Vault

    In this case, the existing library infrastructure would probably need significant changes to allow a Vault to act as an MPID Client (e.g. the MPID struct is defined in the SAFE Client library).

- Another point is that (as with MAID accounts), there is no cleanup done by the network of `MpidMessage`s if the user decides to stop using SAFE.

- Also, not exactly within the scope of this RFC, but related to it; MPID packets at the moment have no human-readable name. It would be more user-friendly to provide this functionality.

## Appendix

#### Further Implementation Details

All MPID-related messages will be in the form of a Put, Post or Delete of `PlainData`.

Such `PlainData` will be:

```rust

PlainData {
    name: mpid_message_name(mpid_message),
    value: serialise::(MpidMessageWrapper),
}

```

The various different types for `PlainData::value` can be enumerated as:

```rust
#[allow(variant_size_differences)]
enum MpidMessageWrapper {
    /// Notification that the MPID Client has just connected to the network
    Online,
    /// Send out an MpidMessage
    PutMessage(MpidMessage),
    /// Send out an MpidHeader and sender's original Authority and SignedToken to allow response
    /// to sender's Put
    PutHeader(MpidHeader, Authority, Option<SignedToken>),
    /// Try to retrieve the message corresponding to the included header
    GetMessage(MpidHeader),
    /// List of headers to check for continued existence of corresponding messages in Sender's outbox
    OutboxHas(Vec<MpidHeader.name()>),
    /// Subset of list from Has request which still exist in Sender's outbox
    OutboxHasResponse(Vec<MpidHeader>),
    /// Retrieve the list of headers of all messages in Sender's outbox
    GetOutboxHeaders,
    /// The list of headers of all messages in Sender's outbox
    GetOutboxHeadersResponse(Vec<MpidHeader>),
}
```

The following list of all MPID-related messages show how this enum is used. (In these tables, Client(A) is the original sender, Client(B) is the recipient, and Managers(A) and Managers(B) are their respective `MpidManager`s).

Requests composed by Client:

| Request | Usage Scenario                                                      | Content                                | Destination Authority           |
|:--------|:--------------------------------------------------------------------|:---------------------------------------|:--------------------------------|
| Put     | Client(A) creating a new message                                    | `Wrapper::MpidMessage`                 | Managers(A)                     |
| Post    | Client(A) checking existence of list of sent messages in own outbox | `Wrapper::OutboxHas(Vec<Header.name>)` | Managers(A)                     |
| Post    | Client(A) getting list of all messages still in own outbox          | `Wrapper::GetOutboxHeaders`            | Managers(A)                     |
| Delete  | Client(A) or (B) deleting from own outbox or inbox respectively     | `XorName`                              | Managers(A) or (B) respectively |
| Delete  | Client(B) deleting a "read" message from sender's outbox            | `XorName`                              | Managers(A)                     |
| Post    | Client announcing to Managers it's connected to network             | `Wrapper::Online`                      | Managers                        |

Requests composed by `MpidManager`:

| Request      | Usage Scenario                                             | Content                                                 | From Authority | Destination Authority |
|:-------------|:-----------------------------------------------------------|:--------------------------------------------------------|:---------------|:----------------------|
| PutResponse  | put failure (outbox or inbox full)                         | `Error(Wrapper::MpidMessage)`                           | Managers(A)    | Client(A)             |
| Put          | new message notification                                   | `Wrapper::MpidHeader`                                   | Managers(A)    | Managers(B)           |
| PutResponse  | put failure (inbox full)                                   | `Error(Wrapper::MpidHeader)`                            | Managers(B)    | Managers(A)           |
| Post         | retrieve message from sender's outbox to pass to recipient | `Wrapper::GetMessage(Header.name)`                      | Managers(B)    | Managers(A)           |
| PostResponse | requested message no longer available in sender's outbox   | `Error(Wrapper::GetMessage(` `Header.name))`            | Managers(A)    | Managers(B)           |
| Post         | pass message from sender's outbox to recipient's managers  | `Wrapper::MpidMessage`                                  | Managers(A)    | Managers(B)           |
| Post         | push message to intended recipient                         | `Wrapper::MpidMessage`                                  | Managers(B)    | Client(B)             |
| Post         | reply to client's `OutboxHas` request                      | `Wrapper::OutboxHasResponse(` `Vec<MpidHeader>)`        | Managers(A)    | Client(A)             |
| Post         | reply to client's `GetOutboxHeaders` request               | `Wrapper::GetOutboxHeadersResponse(` `Vec<MpidHeader>)` | Managers(A)    | Client(A)             |

MPID Header:

```rust
/// Maximum allowed size for `MpidHeader::metadata`
pub const MAX_HEADER_METADATA_SIZE: usize = 128;  // bytes

const GUID_SIZE: usize = 16;

/// MpidHeader
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, RustcDecodable, RustcEncodable)]
pub struct MpidHeader {
    sender_name: XorName,
    guid: [u8; GUID_SIZE],
    metadata: Vec<u8>,
    signature: ::sodiumoxide::crypto::sign::Signature,
}

impl MpidHeader {
    pub fn new(sender_name: XorName,
               metadata: Vec<u8>,
               secret_key: &::sodiumoxide::crypto::sign::SecretKey)
               -> Result<MpidHeader, ::error::RoutingError> {
        use rand::Rng;
        if metadata.len() > MAX_HEADER_METADATA_SIZE {
            return Err(::error::RoutingError::ExceededBounds);
        }
        let mut guid = [0u8; GUID_SIZE];
        ::rand::thread_rng().fill_bytes(&mut guid);

        let encoded = Self::encode(&sender_name, &guid, &metadata);
        Ok(MpidHeader{
            sender_name: sender_name,
            guid: guid,
            metadata: metadata,
            signature: ::sodiumoxide::crypto::sign::sign_detached(&encoded, secret_key),
        })
    }

    pub fn sender_name(&self) -> &XorName {
        &self.sender_name
    }

    pub fn guid(&self) -> &[u8; GUID_SIZE] {
        &self.guid
    }

    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }

    pub fn signature(&self) -> &::sodiumoxide::crypto::sign::Signature {
        &self.signature
    }

    pub fn verify(&self, public_key: &::sodiumoxide::crypto::sign::PublicKey) -> bool {
        let encoded = Self::encode(&self.sender_name, &self.guid, &self.metadata);
        ::sodiumoxide::crypto::sign::verify_detached(&self.signature, &encoded, public_key)
    }

    fn encode(sender_name: &XorName, guid: &[u8; GUID_SIZE], metadata: &Vec<u8>) -> Vec<u8> {
        ::utils::encode(&(sender_name, guid, metadata)).unwrap_or(vec![])
    }
}
```

MPID Message:

```rust
/// Maximum allowed size for `MpidMessage::body`.
pub const MAX_BODY_SIZE: usize = 102400 - 512 -
                                 ::mpid_header::MAX_HEADER_METADATA_SIZE;

/// MpidMessage
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, RustcDecodable, RustcEncodable)]
pub struct MpidMessage {
    header: MpidHeader,
    recipient: XorName,
    body: Vec<u8>,
    recipient_and_body_signature: ::sodiumoxide::crypto::sign::Signature,
}

impl MpidMessage {
    pub fn new(header: MpidHeader,
               recipient: XorName,
               body: Vec<u8>,
               secret_key: &::sodiumoxide::crypto::sign::SecretKey)
               -> Result<MpidMessage, ::error::RoutingError> {
        if body.len() > MAX_BODY_SIZE {
            return Err(::error::RoutingError::ExceededBounds);
        }

        let recipient_and_body = Self::encode(&recipient, &body);
        Ok(MpidMessage {
            header: header,
            recipient: recipient,
            body: body,
            recipient_and_body_signature:
                ::sodiumoxide::crypto::sign::sign_detached(&recipient_and_body, secret_key),
        })
    }

    pub fn header(&self) -> &MpidHeader {
        &self.header
    }

    pub fn recipient(&self) -> &XorName {
        &self.recipient
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn verify(&self, public_key: &::sodiumoxide::crypto::sign::PublicKey) -> bool {
        let encoded = Self::encode(&self.recipient, &self.body);
        ::sodiumoxide::crypto::sign::verify_detached(&self.recipient_and_body_signature, &encoded,
                                                     public_key) && self.header.verify(public_key)
    }

    fn encode(recipient: &XorName, body: &Vec<u8>) -> Vec<u8> {
        ::utils::encode(&(recipient, body)).unwrap_or(vec![])
    }
}
```

Structs in MpidManager to holding the account and messages:

```rust
pub struct MpidManager {
    // key: account owner's mpid_name; value: account
    accounts: HashMap<XorName, Account>,
    chunk_store_inbox: ChunkStore,
    chunk_store_outbox: ChunkStore,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
struct Account {
    // account owners' registerred client proxies
    clients: Vec<::routing::Authority::Client>,
    inbox: MailBox,
    outbox: MailBox,
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Clone)]
struct MailBox {
    allowance: u64,
    used_space: u64,
    space_available: u64,
    // key: msg or header's name; value: sender's public key
    mail_box: HashMap<XorName, PublicKey>,
}
```

Pseudo-code for `MpidManager`:

```rust
impl MpidManager {
    // sending message:
    //     1, messaging: put request from sender A to its MpidManagers(A)
    //     2, notifying: from MpidManagers(A) to MpidManagers(B)
    pub fn handle_put(from, to, pd, token) {
        if messaging {  // pd.value holds MpidMessage
            // insert received mpid_message into the outbox_storage
            if outbox_storage.insert(from, mpid_message) {
                let forward_pd = PlainData {
                    name: mpid_message_name(mpid_message),
                    value: serialise(MpidMessageWrapper::PutHeader(
                            mpid_message.mpid_header,
                            ::mpid_manager::Authority(mpid_message.mpid_header.sender_name())),
                }
                routing.put_request(::mpid_manager::Authority(mpid_message.recipient), forward_pd);
            } else {
                // outbox full or other failure
                reply failure to the sender (Client);
            }
        }
        if notifying {  // pd.value holds MpidHeader
            // insert received mpid_header into the inbox_storage
            if inbox_storage.insert(to, mpid_header) {
                let recipient_account = inbox_storage.find_account(to);
                if recipient_account.recipient_clients.len() > 0 { // indicates there is connected client
                    for header in recipient_account.headers {
                        get_message(header);
                    }
                }
            } else {
                // inbox full or other failure
                reply failure to the sender (MpidManagers);
            }
        }
    }

    // get messages or headers on request:
    pub fn handle_get(from, to, name, token) {
        if outbox.has_account(name) {
            // sender asking for the headers of existing messages
            reply to the requester(from) with outbox.find_account(name).get_headers() via routing.post;
        }
        if inbox.has_account(name) {
            // triggering pushing all existing messages to client, first needs to fetch them
            let recipient_account = inbox_storage.find_account(to);
            if recipient_account.recipient_clients.len() > 0 { // indicates there is connected client
                for header in recipient_account.headers {
                    get_message(header);
                }
            }
        }
    }

    // removing message or header on request:
    //     1, remove_message: delete request from recipient B to sender's MpidManagers(A)
    //     2, remove_header: delete request from recipient B to MpidManagers(B)
    pub fn handle_delete(from, to, name) {
        if remove_message {  // from.name != to.name
            remove the message (bearing the name) from sender(to.name)'s account if the message's
            specified recipient is the requester (from);
        }
        if remove_header {  // from.name == to.name
            remove the header (bearing the name) from recipient(from.name)'s account;
        }
    }

    // handle_post:
    //     1, register_online: client sends a POST request to claim it is online
    //     2, replying: MpidManager(A) forward full message to MpidManager(B) on request
    //     3, fetching: MpidManager(B) trying to fetch a message from MpidManager(A)
    pub fn handle_post(from, to, sd) {
        if register_online {
            let (mpid_name, mpid_client) = (to.name, from);
            let mut recipient_account = inbox.find_account(mpid_name);
            recipient_account.register_online(mpid_client);
            for header in recipient_account.headers {
                send a get request to the sender's MpidManager asking for the full message;
            }
            let mut sender_account = outbox.find_account(mpid_name);
            sender_account.register_online(mpid_client);
        }
        if replying {  // MpidManager(A) replies to MpidManager(B) with the requested mpid_message
            let account = inbox.find_account(to_name);
            if account.has_header(mpid_message.name()) {
                forward the mpid_message to client via routing.post using (reply_to, token);
            }
        }
        if fetching {
            if outbox.has_message(name) {
                // recipient's MpidManager asking for a particular message and it exists
                if the requester is the recipient, reply message to the requester(from) with
                outbox.find_message(name) via routing.post;
            } else {
                // recipient's MpidManager asking for a particular message but not exists
                reply failure to the requester(from);
            }
        }
    }

    // handle_post_response:
    //     1, no_record: response contains Error msg holding the original get request which has
    //        original_mpid_header_name
    //     2, inbox_full: response contains Error msg holding the original put request which has
    //        original_mpid_header
    pub fn handle_post_failure(from, to, response) {
        if no_record {
            // MpidManager(A) replies to MpidManager(B) that the requested mpid_message doesn't exist
            remove the header (bearing the original_mpid_header_name) from the account of to.name;
        }
        if inbox_full {  // MpidManager(B) replies to MpidManager(A) that inbox is full
            remove the message (bearing the original_mpid_header.name()) from the account of to.name;
            original sender's `reply_to` and `token` will be available in this incoming message
            send failure to client via routing.put_failure using (reply_to, token, message);
        }
    }

    fn get_message(header: (sender_name: XorName,
                            sender_public_key: ::sodiumoxide::crypto::sign::PublicKey,
                            mpid_header: MpidHeader)) {
        let request_pd = PlainData {
            name: mpid_header.sender_name,
            value: seialise(MpidMessgeWrapper::GetMessage(mpid_header_name(mpid_header))),
        }
        routing.post_request(::mpid_manager::Authority(header.sender_name), request_pd);
    }

}

```

Pseudo-code for MpidClient:

```rust
/// Client send out an mpid_message to recipient
pub fn send_mpid_message(my_mpid: Mpid, recipient: ::routing::Authority::Client,
                         metadata: Vec<u8>, body: Vec<u8>) {
    let mpid_message = MpidMessage::new(my_mpid, recipient, metadata: Vec<u8>, body);
    let sd = PlainData {
        name: mpid_message_name(mpid_message),
        data: serialise(MpidMessageWrapper::PutMessage(mpid_message)),
    }
    client_routing.put_request(::mpid_manager::Authority(my_mpid.name),
                               ::routing::data::Data::PlainData(pd));
}

/// Client register to be online
pub fn register_online(my_mpid: Mpid) {
    let pd = PlainData {
        name: my_mpid.name,
        value: serialise(MpidMessageWrapper::Online),
    }
    client_routing.post_request(::mpid_manager::Authority(my_mpid.name),
                                ::routing::data::Data::PlainData(pd));
}

/// Client get all headers:
///     1, headers of existing messages in Outbox returned as a Post from vault
///     2, messages of existing headers in Inbox returned as seperate Posts
pub fn get_headers(my_mpid: Mpid) {
    routing.get_request(::sd_manager::Authority(my_mpid.name),
                        ::routing::data::DataRequest::StructuredData(my_mpid.name, 0));
}

pub fn delete_message(my_mpid: Mpid, mpid_header: MpidHeader) {
    client_routing.delete_request(::mpid_manager::Authority(mpid_header.sender), mpid_header.name());
}

fn on_post(sd) {
    match parse_what_inside_sd(sd.data) {
        MpidMessage(mpid_message) => {
            // Client as recipient receiving an incoming message
            handle receiving an incoming message;
            delete_message(my_mpid, mpid_message.mpid_header);
            delete_header(my_mpid, mpid_message.name());
        }
        Outbox(mpid_headers) => {
            // Client as sender receiving a header list of existing messages
            handle receiving a header list of existing messages;
        }
    }
}

fn on_put_failure() {
    handle_failure;
}

fn on_post_failure() {
    handle_failure;
}

```

General functions

```rust
pub fn mpid_header_name(mpid_header: &MpidHeader) -> XorName {
    ::crypto::hash::sha512::hash(::utils::encode(mpid_header))
}
pub fn mpid_message_name(mpid_message: &MpidMessage) -> XorName {
    mpid_header_name(mpid_message.mpid_header)
}
```



[0]: https://github.com/maidsafe/safe_client/blob/c4dbca0e5ee8122a6a7e9441c4dcb65f9ef96b66/src/client/user_account.rs#L27-L29
[1]: #mpidheader
[2]: #mpidmessage
[3]: #outbox
[4]: #inbox
[5]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/plain_data.rs
[6]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/data.rs#L24-L33
