- Feature Name: MPID Messaging System
- Type: New Feature
- Related Components: [safe_vault](https://github.com/maidsafe/safe_vault), [safe_client](https://github.com/maidsafe/safe_client), [routing](https://github.com/maidsafe/routing)
- Start Date: 22-09-2015
- RFC PR:
- Issue number:

# Summary

This RFC outlines the system componenets and design for general communications infrastructure and security on the SAFE Network.

# Motivation

## Rationale

A messaging system on the SAFE Network will obviously be useful.  Over and above this, the propsed messaging system is secure and private without the possibility of snooping or tracking.

A large abuse of modern day digital communications is the ability of spammers and less than ethical marketing companies to flood the Internet with levels of unsolicited email to a level of circa 90%.  This is a waste of bandwidth and a significant nuisance.

## Supported Use-Cases

The system will support secure, private, bi-directional communications between pairs of Clients.

## Expected Outcome

The provision of a secure messaging system which will eradicate unwanted and intrusive communications.

# Detailed design

## Overview

A fundamental principle is that the cost of messaging is with the sender, primarily to deter unsolicited messages.  To achieve this, the sender will maintain messages in a network [outbox][3] until they are retrieved by the recipient.  The recipient will have a network [inbox][4] comprising a list of metadata relating to messages which are trying to be delivered to it.

If the message is unwanted, the recipient simply does not retrieve the message.  The sender will thus quickly fill their own outbox with undelivered mail and be forced to clean this up, themselves before being able to send further messages.

This paradigm shift will mean that the obligation to unsubscribe from mailing lists, etc. is now with the owner of these lists. If people are not picking up mail, it is because they do not want it.  So the sender has to do a better job.  It is assumed this shift of responsibilities will lead to a better managed bandwidth solution and considerably less stress on the network and the users of the network.

## Implementation Details

The two relevant structs (other than the MPID itself which is a [standard Client key][0]) are the [`MpidHeader`][1] and the [`MpidMessage`][2].

Broadly speaking, the [`MpidHeader`][1] contains metadata and the [`MpidMessage`][2] contains a signed header and message.  We also want to define some upper limits which will be described later.

### Consts

```rust
pub const MPID_MESSAGE_TAG: u64 = 51000;
pub const MPID_HEADER_TAG: u64 = 51001;
pub const MAX_SUBJECT_SIZE: usize = 128;  // bytes
pub const MAX_INBOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB
pub const MAX_OUTBOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB
```

### `MpidHeader`

```rust
pub struct MpidHeader {
    pub sender: ::routing::NameType,
    pub recipient: ::routing::NameType,
    pub index: u32,
    pub parent_index: Option<u32>,
    pub subject: Vec<u8>,
}
```

The `sender` and `recipient` fields are hopefully self-explanatory.  However, the only purpose for them existing in the `MpidHeader` is to increase the randomness of the derived message name so conflicts have a much lower chance of happening.

The `index` and `parent_index` are intended to allow threading or sequencing of messages, while `subject` allows passing arbitrary data (e.g. a subject line).

The `subject` field must not exceed `MAX_SUBJECT_SIZE` bytes.

### `MpidMessage`

```rust
pub struct MpidMessage {
    pub signed_header: Vec<u8>,
    pub signed_body: Vec<u8>,
}
```
Each `MpidMessage` instance only targets one recipient.  For multiple recipients, multiple `MpidMessage`s need to be created in the [Outbox][3] (see below).  This is to ensure spammers will run out of limited resources quickly.

### Functions

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
    decode::<MpidHeader>(::sodiumoxide::crypto::sign::verify(signed_header, public_key))
}
```

## Outbox

This is a simple data structure for now and will be a hash map of serialised and encrypted `MpidMessage`s.  There will be one such map per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

## Inbox

This is an even simpler structure and again there will be one per MPID (owner), held on the MpidManagers, and synchronised by them at churn events.

This can be implemented as a `Vec<(sender: ::routing::NameType, signed_header: Vec<u8>)>`.

## Messaging Format Among Nodes

Messages between Clients and MpidManagers will utilise [`::routing::structured_data::StructuredData`][5]:
```rust
let sd_for_mpid_message = StructuredData {
    type_tag: MPID_MESSAGE_TAG,
    identifier: mpid_message_name(mpid_message),
    data: encode(mpid_message),
    previous_owner_keys: vec![],
    version: 0,
    current_owner_keys: vec![mpid_message.sender],
    previous_owner_signatures: vec![]
}
```
```rust
let sd_for_mpid_header = StructuredData {
    type_tag: MPID_HEADER_TAG,
    identifier: mpid_message_name(mpid_message),  // or mpid_header_name(signed_header)
    data: mpid_message.signed_header,
    previous_owner_keys: vec![],
    version: 0,
    current_owner_keys: vec![mpid_message.sender],  // or vec![mpid_header.sender]
    previous_owner_signatures: vec![]
}
```

## Message Flow

```
                    MpidManagers(A)    MpidManagers(B)
                   /      *                   *        \
MpidClient(A) ->  -       *                   *         -  <- MpidClient(B)
                   \      *                   *        /

```
1. `MpidClient(A)` sends an `MpidMessage` to `MpidManagers(A)` as a Put request.
1. `MpidManagers(A)` store this message in their outbox for A and send the `signed_header` component to `MpidManagers(B)` as a Put request, unless the outbox is full in which case a PutFailure response is returned to `MpidClient(A)`.
1. `MpidManagers(B)` try to store the `(sender, signed_header)` in their inbox for B.  If successful, they forward it to `MpidClient(B)` as a Put request immediately or as soon as it appears online.  If unsuccessful (e.g. the inbox is full or sender has been blacklisted), they reply to `MpidManagers(A)` with a PutFailure, who then remove the message from the outbox and send a PutFailure response to `MpidClient(A)`.
1. To retrieve the message from the network, `MpidClient(B)` sends a Get request to `MpidManagers(B)`, which is forwarded to `MpidManagers(A)`.
1. `MpidManagers(A)` send a GetResponse to `MpidManagers(B)` which is forwarded to `MPidClient(B)` if `MPidClient(B)` is online (otherwise it's dropped).
1. On receiving the message, `MpidClient(B)` sends a remove request to `MpidManagers(B)` via a Delete.
1. `MpidManagers(B)` remove the corresponding entry from the inbox for B and forward the remove request to `MpidManagers(A)`.  `MpidManagers(A)` then remove the corresponding entry from the outbox for A.

`MpidClient(A)` might decide to remove the `MpidMessage` from the outbox if the message hasn't been retrieved by `MpidClient(B)` yet.  In this case, `MpidManagers(A)` should not only remove the corresponding `MpidMessage` from their outbox for A, but also send a notification to the group of `MpidManagers(B)` so they can remove the corresponding entry from their inbox of B.  These messages will all be Delete requests.

_MPidClient(A)_ =>> |__MPidManagers(A)__ (Put)(Header.So) *->> | __MPidManagers(B)__  (Store(Header))(Online(MpidClient(B)) ? Header.So : (WaitForOnlineB)(header.So)) *-> | _MpidClient(B)_ So.Retrieve ->> | __MpidManagers(B)__ *-> | __MpidManagers(A)__ So.Message *->> | __MpidManagers(B)__ Online(MpidClient(B)) ? Message.So *-> | _MpidClient(B)_ Remove.So ->> | __MpidManagers(B)__ {Remove(Header), Remove.So} *->> | __MpidManagers(A)__ Remove

## MPID Client

The MPID Client shall provide the following key functionalities :

1. Send Message (Put from sender)
1. Retrieve Full Message (Get from receiver)
1. Remove sent Message (Delete from sender)
1. Accept Message header (if the "push" model is used) and/or Retrieve Message Header (if the "pull" model is used)

If the "push" model is used, an MPID Client is expected to have its own routing object (not shared with the MAID Client).  In this way it can directly connect to its own MpidManagers, allowing them to know its online status and hence they can push message headers to it as and when they arrive.

Such a separate routing object is not required if the "pull" model is used.  This is where the MPID Client periodically polls its network inbox for new headers.  It may also have the benefit of saving the battery life on mobile devices, as the client app doesn't need to keep MPID Client running all the time.

## Planned Work

1, Vault
```
    a, MpidManager::OutBox
    b, MpidManager::InBox
    c, detecting of client, when PUSH model used
    d, sending message flow
    e, retrieving message flow
    f, deleting message flow
    g, churn handling and refreshing for account_transfer
    h, mpid_client addressing (if mpid address registration procedure to be undertaken)
```
2, Routing
```
    a, Authority::MpidManager
    b, PUSH model
        Notifying client with mpid_header when client join
        This is not required if PULL model is used
    c, Definition of MPID_MESSAGE_TAG and MPID_HEADER_TAG
    d, Definition of MpidMessage and MpidHeader
    e, Support Delete (for StructuredData only)
    f, address relocation (if allows client using fixed mpid address connecting network)
        not required if 1.h is implemented
```
3, Client
```
    a, Send Message
    b, Get Message (or Accept Message)
        This shall also includes the work of removing correspondent mpid_headers
    c, Delete Message
    d, address relocation (if allows client using fixed mpid address connecting network)
        not required if 1.h is implemented
```

# Drawbacks

None identified, other than increased complexity of Vault and Client codebase.

# Alternatives

No other in-house alternatives have been documented as yet.

If we were to _not_ implement some form of secure messaging system, a third party would be likely to implement a similar system using the existing interface to the SAFE Network.  This would be unlikely to be as efficient as the system proposed here, since this will be "baked into" the Vault protocols.

We also have identified a need for some form of secure messaging in order to implement safecoin, so failure to implement this RFC would impact on that too.

# Unresolved questions

1. It needs to be mentioned that to figure out the recipient, a `sign::verify` call is required each time.  The efficiency can be improved by having an explicit `recipient` member data in the MpidMessage struct, however this will be in the spacial and bandwidth cost in storage and messaging among nodes.
1.


# Future Work

It might be required to provide Vault-to-Vault, Client-to-Vault ot Vault-to-Client communications in the future.  Potential use cases for this are:

1. Vault-to-Client notification of a successful safecoin mining attempt
1. Client-to-Vault request to take ownership of the Vault
1. Vault-to-Client notification of low resources on the Vault

In this case, the existing library infrastructure would probably need significant changes to allow a Vault to act as an MPID Client (e.g. the MPID struct is defined in the SAFE Client library).





[0]: https://github.com/maidsafe/safe_client/blob/c4dbca0e5ee8122a6a7e9441c4dcb65f9ef96b66/src/client/user_account.rs#L27-L29
[1]: #mpidheader
[2]: #mpidmessage
[3]: #outbox
[4]: #inbox
[5]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/structured_data.rs#L22-L34
