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
pub const MPID_MESSAGE_TAG: u64 = 51000;
pub const MPID_HEADER_TAG: u64 = 51001;
pub const MAX_HEADER_METADATA_SIZE: usize = 128;  // bytes
pub const MAX_BODY_SIZE: usize =
    ::routing::structured_data::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - 512 - MAX_HEADER_METADATA_SIZE;
pub const MAX_INBOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB
pub const MAX_OUTBOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB
```

### `MpidHeader`

```rust
pub struct MpidHeader {
    pub sender_name: ::routing::NameType,
    pub guid: [u8, 16],
    pub metadata: Vec<u8>,
}
```

The `sender` field is hopefully self-explanatory.  The `guid` allows the message to be message to be uniquely identified, both by receivers and by the manager Vaults which need to hold the messages in a map-like structure.

The `metadata` field allows passing arbitrary user/app data.  It must not exceed `MAX_HEADER_METADATA_SIZE` bytes.

### `MpidMessage`

```rust
pub struct RecipientAndBody {
    pub recipient: ::routing::NameType,
    pub body: Vec<u8>,
}

pub struct MpidMessage {
    pub sender_public_key: ::sodiumoxide::crypto::sign::PublicKey;
    pub signed_header: Vec<u8>,
    pub signed_recipient_and_body: Vec<u8>,
}
```
Each `MpidMessage` instance only targets one recipient.  For multiple recipients, multiple `MpidMessage`s need to be created in the [Outbox][3] (see below).  This is to ensure spammers will run out of limited resources quickly.

## Outbox

This is a simple data structure for now and will be a hash map of serialised and encrypted `MpidMessage`s.  There will be one such map per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

This can be implemented as a `Vec<MpidMessage>`.

## Inbox

Again this will be one per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

This can be implemented as a `Vec<(sender_name: ::routing::NameType, sender_public_key: ::sodiumoxide::crypto::sign::PublicKey, signed_header: Vec<u8>)>` or having the headers from the same sender grouped: `Vec<(sender_name: ::routing::NameType, sender_public_key: ::sodiumoxide::crypto::sign::PublicKey, headers: Vec<signed_header: Vec<u8>>)>` (however this may incur a performance slow down when looking up for a particular mpid_header).

## Messaging Format Among Nodes

Messages between Clients and MpidManagers will utilise [`::routing::structured_data::StructuredData`][5], for example:
```rust
let sd_for_mpid_message = StructuredData {
    type_tag: MPID_MESSAGE_TAG,
    identifier: mpid_message_name(mpid_message),
    data: ::utils::encode(mpid_message),
    previous_owner_keys: vec![],
    version: 0,
    current_owner_keys: vec![mpid_message.sender_public_key],
    previous_owner_signatures: vec![]
}
```
```rust
let sd_for_mpid_header = StructuredData {
    type_tag: MPID_HEADER_TAG,
    identifier: mpid_message_name(mpid_message),  // or mpid_header_name(signed_header)
    data: mpid_message.signed_header,  // or inbox.signed_header
    previous_owner_keys: vec![],
    version: 0,
    current_owner_keys: vec![mpid_message.sender_public_key],  // or vec![inbox.sender_public_key]
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

1. `MpidMessage` and `MpidHeader` are wrapped in `StructuredData` instances in order to allow Delete requests on them and to allow them to be given different tags (otherwise they could be ImmutableData, since they don't mutate).  Should Routing, which already has to define these two types, be made "aware" of these two types?  (i.e. should they get added to the [`::routing::data::Data` enum][6])

    Identified pros:
    - less wrapping/parsing, so simpler for Vaults and Clients to deal with
    - more efficient (smaller messages to transmit and store)
    - no need for Routing to publicly expose `MPID_MESSAGE_TAG` or `MPID_HEADER_TAG`

    Identified cons:
    - increased complexity in Routing
    - would need to add the sender's public key to the header Put flow

    Qi prefers using `StructuredData`, Fraser prefers not using `StructuredData` unless the effort required by Routing to accommodate the new types is significant.

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

To send an MPID Message, a client would do something like:

```rust
let mpid_message = MpidMessage::new(my_mpid: Mpid, recipient: ::routing::Authority::Client,
                                    metadata: Vec<u8>, body: Vec<u8>);

```

Account types held by MpidManagers

```rust
struct Outbox {
    pub sender: ::routing::NameType,
    pub mpid_messages: Vec<MpidMessage>,
    pub total_size: u64,
}
struct Inbox {
    pub recipient_name: ::routing::NameType,
    pub recipient_clients: Vec<::routing::Authority::Client>,
    pub headers: Vec<(sender_name: ::routing::NameType,
                      sender_public_key: ::sodiumoxide::crypto::sign::PublicKey,
                      signed_header: Vec<u8>)>,
    pub total_size: u64,
}
```

General functions

```rust
pub fn mpid_header_name(signed_header: &Vec<u8>) -> ::routing::NameType {
    ::crypto::hash::sha512::hash(signed_header)
}
pub fn mpid_message_name(mpid_message: &MpidMessage) -> ::routing::NameType {
    mpid_header_name(mpid_message.signed_header)
}

pub fn sign_mpid_header(mpid_header: &MpidHeader, private_key: PrivateKey) -> Vec<u8> {
    ::sodiumoxide::crypto::sign::sign(encode(mpid_header), private_key)
}
pub fn parse_signed_header(signed_header: &Vec<u8>, public_key: PublicKey) -> Result<MpidHeader, ::routing::error::Error> {
    ::utils::decode::<MpidHeader>(::sodiumoxide::crypto::sign::verify(signed_header, public_key))
}
```



[0]: https://github.com/maidsafe/safe_client/blob/c4dbca0e5ee8122a6a7e9441c4dcb65f9ef96b66/src/client/user_account.rs#L27-L29
[1]: #mpidheader
[2]: #mpidmessage
[3]: #outbox
[4]: #inbox
[5]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/structured_data.rs#L22-L34
[6]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/data.rs#L24-L33
