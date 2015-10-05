- Feature Name: MPID Messaging System
- Type: New Feature
- Related Components: [safe_vault](https://github.com/maidsafe/safe_vault), [safe_client](https://github.com/maidsafe/safe_client), [routing](https://github.com/maidsafe/routing)
- Start Date: 22-09-2015
- RFC PR:
- Issue number:

# Summary

This RFC outlines the system components and design for general communications infrastructure and security on the SAFE Network.

# Motivation

## Rationale

A messaging system on the SAFE Network will obviously be useful.  Over and above this, the proposed messaging system is secure and private without the possibility of snooping or tracking.

A large abuse of modern day digital communications is the ability of spammers and less than ethical marketing companies to flood the Internet with levels of unsolicited email to a level of circa 90%.  This is a waste of bandwidth and a significant nuisance.

## Supported Use-Cases

The system will support secure, private, bi-directional communications between pairs of Clients.

## Expected Outcome

The provision of a secure messaging system which will eradicate unwanted and intrusive communications.

# Detailed design

## Overview

A fundamental principle is that the cost of messaging is with the sender, primarily to deter unsolicited messages.  To achieve this, the sender will maintain messages in a network [outbox][3] until they are retrieved by the recipient.  The recipient will have a network [inbox][4] comprising a list of metadata relating to messages which are trying to be delivered to it.

If the message is unwanted, the recipient simply does not retrieve the message.  The sender will thus quickly fill their own outbox with undelivered mail and be forced to clean this up themselves before being able to send further messages.

This paradigm shift will mean that the obligation to unsubscribe from mailing lists, etc. is now with the owner of these lists.  If people are not picking up messages, it is because they do not want them.  So the sender has to do a better job.  It is assumed this shift of responsibilities will lead to a better-managed bandwidth solution and considerably less stress on the network and the users of the network.

## Implementation Details

The two relevant structs (other than the MPID itself which is a [standard Client key][0]) are the [`MpidHeader`][1] and the [`MpidMessage`][2].

Broadly speaking, the `MpidHeader` contains metadata and the `MpidMessage` contains a signed header and message.  We also want to define some upper limits which will be described later.

### Consts

```rust
pub const MPID_MESSAGE: u64 = 51000;
pub const MAX_HEADER_METADATA_SIZE: usize = 128;  // bytes
pub const MAX_BODY_SIZE: usize =
    ::routing::structured_data::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - 512 - MAX_HEADER_METADATA_SIZE;
pub const MAX_INBOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB
pub const MAX_OUTBOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB
```

### `MpidHeader`

```rust
pub struct MpidHeader {
    sender_name: ::NameType,
    guid: [u8; 16],
    metadata: Vec<u8>,
    signature: ::sodiumoxide::crypto::sign::Signature,
}
```

The `sender` field is hopefully self-explanatory.  The `guid` allows the message to be message to be uniquely identified, both by receivers and by the manager Vaults which need to hold the messages in a map-like structure.

The `metadata` field allows passing arbitrary user/app data.  It must not exceed `MAX_HEADER_METADATA_SIZE` bytes.

### `MpidMessage`

```rust
pub struct MpidMessage {
    header: ::MpidHeader,
    recipient: ::NameType,
    body: Vec<u8>,
    recipient_and_body_signature: ::sodiumoxide::crypto::sign::Signature,
}
```
Each `MpidMessage` instance only targets one recipient.  For multiple recipients, multiple `MpidMessage`s need to be created in the [Outbox][3] (see below).  This is to ensure spammers will run out of limited resources quickly.

## Outbox

This is a simple data structure for now and will be a hash map of serialised and encrypted `MpidMessage`s.  There will be one such map per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

This can be implemented as a `Vec<MpidMessage>`.

## Inbox

Again this will be one per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

This can be implemented as a `Vec<(sender_name: ::routing::NameType, sender_public_key: ::sodiumoxide::crypto::sign::PublicKey, mpid_header: MpidHeader)>` or having the headers from the same sender grouped: `Vec<(sender_name: ::routing::NameType, sender_public_key: ::sodiumoxide::crypto::sign::PublicKey, headers: Vec<mpid_header: MpidHeader>)>` (however this may incur a performance slow down when looking up for a particular mpid_header).

## Messaging Format Among Nodes

Messages between Clients and MpidManagers will utilise [`::routing::structured_data::StructuredData`][5], for example:
```rust
let sd = StructuredData {
    type_tag: MPID_MESSAGE,
    identifier: mpid_message_name(mpid_message),
    data: ::utils::encode(mpid_message),
    previous_owner_keys: vec![],
    version: 0,
    current_owner_keys: vec![sender_public_key],
    previous_owner_signatures: vec![]
}
```

When handling churn, MpidManagers will synchronise their outboxes by sending each included MpidMessage as a single network message.  However, since MpidHeaders are significantly smaller, inboxes will be sent as a message containing a vector of all contained headers.

## Message Flow

![Flowchart showing MPID Message flow through the SAFE Network](MPID%20Message%20Flow.png)

## MPID Client

The MPID Client shall provide the following key functionalities :

1. Send Message (Put from sender)
1. Accept Message header (Push from MpidManagers to recipient)
1. Retrieve Full Message (Get from receiver)
1. Query own inbox to get list of all remaining MpidHeaders
1. Query own inbox for a vector of specific MpidHeaders to see whether they remain in the outbox or not.
1. Remove unwanted MpidHeader (Delete from recipient)
1. Query own outbox to get list of all remaining MpidMessages
1. Remove sent Message (Delete from sender)

If the "push" model is used, an MPID Client is expected to have its own routing object (not shared with the MAID Client).  In this way it can directly connect to its own MpidManagers (or the connected ClientManager will register itself as the proxy to the corresponding MpidManagers), allowing them to know its online status and hence they can push message headers to it as and when they arrive.

Such a separate routing object (or the registering procedure) is not required if the "pull" model is employed.  This is where the MPID Client periodically polls its network inbox for new headers.  It may also have the benefit of saving the battery life on mobile devices, as the client app doesn't need to keep MPID Client running all the time.

## Planned Work

1. Vault
    1. outbox
    1. inbox
    1. sending message flow
    1. retrieving message flow
    1. deleting message flow
    1. churn handling and refreshing for account_transfer (Inbox and Outbox)
    1. MPID Client registering (when GetAllHeader request received)

1. Routing
    1. `Authority::MpidManager`
    1. definition of `MPID_MESSAGE_TAG` and `MPID_HEADER_TAG`
    1. definition of `MpidMessage` and `MpidHeader`
    1. support Delete (for StructuredData only)
    1. support push to client

1. Client
    1. Put `MpidMessage`
    1. Get all `MpidHeader`s (pull)
    1. accept all/single `MpidHeader` (push)
    1. Get `MpidMessage`.  This shall also include the work of removing corresponding `MpidHeader`s
    1. Delete `MpidMessage`
    1. Delete `MpidHeader`


# Drawbacks

None identified, other than increased complexity of Vault and Client codebase.

# Alternatives

No other in-house alternatives have been documented as yet.

If we were to _not_ implement some form of secure messaging system, a third party would be likely to implement a similar system using the existing interface to the SAFE Network.  This would be unlikely to be as efficient as the system proposed here, since this will be "baked into" the Vault protocols.

We also have identified a need for some form of secure messaging in order to implement safecoin, so failure to implement this RFC would impact on that too.

# Unresolved Questions


# Future Work

- This RFC doesn't address how Clients get to "know about" each other.  Future work will include details of how Clients can exchange MPIDs in order to build an address book of peers.

- It might be required to provide Vault-to-Vault, Client-to-Vault or Vault-to-Client communications in the future.  Potential use cases for this are:

    1. Vault-to-Client notification of a successful safecoin mining attempt
    1. Client-to-Vault request to take ownership of the Vault
    1. Vault-to-Client notification of low resources on the Vault

    In this case, the existing library infrastructure would probably need significant changes to allow a Vault to act as an MPID Client (e.g. the MPID struct is defined in the SAFE Client library).

- Another point is that (as with MAID accounts), there is no cleanup done by the network of MpidMessages if the user decides to stop using SAFE.

- Also, not exactly within the scope of this RFC, but related to it; MPID packets at the moment have no human-readable name.  It would be more user-friendly to provide this functionality.

# Appendix

### Further Implementation Details

All MPID-related messages will be in the form of a Put, Post or Delete of a `StructuredData`.

This will be:

```rust

StructuredData {
    type_tag: MPID_MESSAGE,
    identifier: mpid_message_name(mpid_message),  // or mpid_header_name(mpid_header)
    data: XXX,
    previous_owner_keys: vec![],
    version: 0,
    current_owner_keys: vec![sender_public_key],
    previous_owner_signatures: vec![],
}

```

where XXX is indicated in the following list of all MPID-related messages:

Sent from Client:

| Message | From ==> To |
|:---|:---|
| Post[Online] |           Either Client    ==> Own Managers |
| Put[Message] |           Sender Client    ==> Own Managers |
| Post[Has[Vec<Header>]] | Sender Client    ==> Own Managers |
| Post[GetAllHeaders] |    Sender Client    ==> Own Managers |
| Delete[Header] |         Either Client    ==> Own Managers (try to delete from inbox and outbox) |
| Delete[Header] |         Recipient Client ==> Sender's Managers |


Sent from Vault:

| Message | From ==> To |
|:---|:---|
| PutResponse[Error[Message]] |              Sender's Managers (outbox)   ==> Sender Client |
| Put[Header] |                              Sender's Managers (outbox)   ==> Recipient's Managers (inbox) |
| PutResponse[Error[Header]] |               Recipient's Managers (inbox) ==> Sender's Managers (outbox) |
| Post[GetMessage[Header]] |                 Recipient's Managers (inbox) ==> Sender's Managers (outbox) |
| PostResponse[Error[GetMessage[Header]]] |  Sender's Managers (outbox)   ==> Recipient's Managers (inbox) |
| Post[Message] |                            Sender's Managers (outbox)   ==> Recipient's Managers (inbox) |
| Post[Message] |                            Recipient's Managers (inbox) ==> Recipient Client |
| Post[HasResponse[Vec<Header>]] |           Sender's Managers (outbox)   ==> Sender Client |
| Post[GetAllHeadersResponse[Vec<Header>]] | Sender's Managers (outbox)   ==> Sender Client |


The various different types for `StructuredData::data` can be enumerated as:

```rust
#[allow(variant_size_differences)]
enum MpidMessageWrapper {
    /// Notification that the MPID Client has just connected to the network
    Online,
    /// Try to retrieve the message corresponding to the included header
    GetMessage(MpidHeader),
    /// List of headers to check for continued existence of corresponding messages in Sender's outbox
    Has(Vec<MpidHeader>),
    /// Subset of list from Has request which still exist in Sender's outbox
    HasResponse(Vec<MpidHeader>),
    /// Retrieve the list of headers of all messages in Sender's outbox
    GetAllHeaders,
    /// The list of headers of all messages in Sender's outbox
    GetAllHeadersResponse(Vec<MpidHeader>),
}
```


MPID Header:

```rust
/// Maximum allowed size for `MpidHeader::metadata`
pub const MAX_HEADER_METADATA_SIZE: usize = 128;  // bytes

const GUID_SIZE: usize = 16;

/// MpidHeader
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, RustcDecodable, RustcEncodable)]
pub struct MpidHeader {
    sender_name: ::NameType,
    guid: [u8; GUID_SIZE],
    metadata: Vec<u8>,
    signature: ::sodiumoxide::crypto::sign::Signature,
}

impl MpidHeader {
    pub fn new(sender_name: ::NameType,
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

    pub fn sender_name(&self) -> &::NameType {
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

    fn encode(sender_name: &::NameType, guid: &[u8; GUID_SIZE], metadata: &Vec<u8>) -> Vec<u8> {
        ::utils::encode(&(sender_name, guid, metadata)).unwrap_or(vec![])
    }
}
```

MPID Message:

```rust
/// Maximum allowed size for `MpidMessage::body`.
pub const MAX_BODY_SIZE: usize = ::structured_data::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - 512 -
                                 ::mpid_header::MAX_HEADER_METADATA_SIZE;

/// MpidMessage
#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, RustcDecodable, RustcEncodable)]
pub struct MpidMessage {
    header: ::MpidHeader,
    recipient: ::NameType,
    body: Vec<u8>,
    recipient_and_body_signature: ::sodiumoxide::crypto::sign::Signature,
}

impl MpidMessage {
    pub fn new(header: ::MpidHeader,
               recipient: ::NameType,
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

    pub fn header(&self) -> &::MpidHeader {
        &self.header
    }

    pub fn recipient(&self) -> &::NameType {
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

    fn encode(recipient: &::NameType, body: &Vec<u8>) -> Vec<u8> {
        ::utils::encode(&(recipient, body)).unwrap_or(vec![])
    }
}
```

Account types held by MpidManagers

```rust
struct OutboxAccount {
    pub sender: ::routing::NameType,
    pub sender_clients: Vec<::routing::Authority::Client>,
    pub mpid_messages: Vec<MpidMessage>,
    pub total_size: u64,
}
struct InboxAccount {
    pub recipient_name: ::routing::NameType,
    pub recipient_clients: Vec<::routing::Authority::Client>,
    pub headers: Vec<(sender_name: ::routing::NameType,
                      sender_public_key: ::sodiumoxide::crypto::sign::PublicKey,
                      mpid_header: MpidHeader)>,
    pub total_size: u64,
}
```

Pseudo-code for MpidManager:

```rust
pub struct MpidManager {
    outbox_storage: Vec<OutboxAccount>,
    inbox_storage: Vec<InboxAccount>,
    routing: Routing,
}

impl MpidManager {
    // sending message:
    //     1, messaging: put request from sender A to its MpidManagers(A)
    //     2, notifying: from MpidManagers(A) to MpidManagers(B)
    pub fn handle_put(from, to, sd) {
        if messaging {  // sd.data holds MpidMessage
            // insert received mpid_message into the outbox_storage
            if outbox_storage.insert(from, mpid_message) {
                let forward_sd = StructuredData {
                    type_tag: MPID_MESSAGE,
                    identifier: mpid_message_name(mpid_message),
                    data: ::utils::encode(mpid_message.mpid_header),
                    previous_owner_keys: vec![],
                    version: 0,
                    current_owner_keys: vec![my_mpid.public_key],
                    previous_owner_signatures: vec![]
                }
                routing.put_request(::mpid_manager::Authority(mpid_message.recipient), forward_sd);
            } else {
                // outbox full or other failure
                reply failure to the sender (Client);
            }
        }
        if notifying {  // sd.data holds MpidHeader
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
    pub fn handle_get(from, to, name) {
        if out_box.has_account(name) {
            // sender asking for the headers of existing messages
            reply to the requester(from) with out_box.find_account(name).get_headers() via routing.post;
        }
        if in_box.has_account(name) {
            // triggering pushing all existing messages to client, first needs to fetch them
            let recipient_account = inbox_storage.find_account(to);
            if recipient_account.recipient_clients.len() > 0 { // indicates there is connected client
                for header in recipient_account.headers {
                    get_message(header);
                }
            }
        }
    }
    
    // revoming message or header on request:
    //     1, remove_message: delete request from recipient B to sender's MpidManagers(A)
    //     2, remove_header: delete request from recipient B to MpidManagers(B)
    pub fn handle_delete(from, to, name) {
        if remove_message {  // from.name != to.name
            remove the message (bearing the name) from sender(to.name)'s account if the message's specified recipient is the requester (from);
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
            sender_account.register_on_line(mpid_client);
        }
        if replying {  // MpidManager(A) replies to MpidManager(B) with the requested mpid_message
            let account = inbox.find_account(to_name);
            if account.has_header(mpid_message.name()) {
                for client in account.recipient_clients {
                    foward the mpid_message to client via routing.post;
                }
            }
        }
        if fetching {
            if out_box.has_message(name) {
                // recipient's MpidManager asking for a particular message and it exists
                if the requester is the recipient, reply message to the requester(from) with out_box.find_message(name) via routing.post;
            } else {
                // recipient's MpidManager asking for a particular message but not exists
                reply failure to the requester(from);
            }
        }
    }

    // handle_post_response:
    //     1, no_record: response contains Error msg holding the original get request which has ori_mpid_header_name
    //     2, inbox_full: response contains Error msg holding the original put request which has ori_mpid_header
    pub fn handle_post_failure(from, to, response) {
        if no_record {  // MpidManager(A) replies to MpidManager(B) that the requested mpid_message doesn't exists
            remove the header (bearing the ori_mpid_header_name) from the account of to.name;
        }
        if inbox_full {  // MpidManager(B) replies to MpidManager(A) that inbox is full
            remove the message (bearing the ori_mpid_header.name()) from the account of to.name;
            send failure to each sender_client in outbox.find_account(to.name).sender_clients via routing.post_response;
        }
    }
    
    fn get_message(header: (sender_name: ::routing::NameType,
                            sender_public_key: ::sodiumoxide::crypto::sign::PublicKey,
                            mpid_header: MpidHeader)) {
        let request_sd = StructuredData {
            type_tag: MPID_MESSAGE,
            identifier: header.sender_name,
            data: ::utils::encode(MpidMessgeWrapper::GetMessage(mpid_header_name(mpid_header))),
            previous_owner_keys: vec![],
            version: 0,
            current_owner_keys: vec![header.sender_public_key],
            previous_owner_signatures: vec![]
        }
        routing.post_request(::mpid_manager::Authority(header.sender_name), request_sd);
    }

}

```

Pseudo-code for MpidClient:

```rust
/// Client send out an mpid_message to recipient
pub fn send_mpid_message(my_mpid: Mpid, recipient: ::routing::Authority::Client,
                         metadata: Vec<u8>, body: Vec<u8>) {
    let mpid_message = MpidMessage::new(my_mpid, recipient, metadata: Vec<u8>, body);
    let sd = StructuredData {
        type_tag: MPID_MESSAGE,
        identifier: mpid_message_name(mpid_message),
        data: ::utils::encode(mpid_message),
        previous_owner_keys: vec![],
        version: 0,
        current_owner_keys: vec![my_mpid.public_key],
        previous_owner_signatures: vec![]
    }
    client_routing.put_request(::mpid_manager::Authority(my_mpid.name),
                               ::routing::data::Data::StructuredData(sd));
}

/// Client register to be on-line
pub fn register_online(my_mpid: Mpid) {
    let sd = StructuredData {
        type_tag: MPID_MESSAGE,
        identifier: my_mpid.name,
        data: ::utils::encode(MpidMessageWrapper::Online),,
        previous_owner_keys: vec![],
        version: 0,
        current_owner_keys: vec![my_mpid.public_key],
        previous_owner_signatures: vec![]
    }
    client_routing.post_request(::mpid_manager::Authority(my_mpid.name),
                                ::routing::data::Data::StructuredData(sd));
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
pub fn mpid_header_name(mpid_header: &MpidHeader) -> ::routing::NameType {
    ::crypto::hash::sha512::hash(::util::encode(mpid_header))
}
pub fn mpid_message_name(mpid_message: &MpidMessage) -> ::routing::NameType {
    mpid_header_name(mpid_message.mpid_header)
}
```



[0]: https://github.com/maidsafe/safe_client/blob/c4dbca0e5ee8122a6a7e9441c4dcb65f9ef96b66/src/client/user_account.rs#L27-L29
[1]: #mpidheader
[2]: #mpidmessage
[3]: #outbox
[4]: #inbox
[5]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/structured_data.rs#L22-L34
[6]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/data.rs#L24-L33
