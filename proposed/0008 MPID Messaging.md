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
    current_owner_keys: vec![mpid_message.sender],  // or vec![inbox.sender]
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
1. To retrieve the message from the network, `MpidClient(B)` sends a Get request to `MpidManagers(A)`.
1. `MpidManagers(A)` send a GetResponse to `MPidClient(B)`.
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

1. Vault
    1. outbox
    1. inbox
    1. detecting of Client, if "push" model used
    1. sending message flow
    1. retrieving message flow
    1. deleting message flow
    1. churn handling and refreshing for account_transfer
    1. MPID Client addressing (if MPID address registration procedure is to be undertaken - i.e. in "pull" model)

1. Routing
    1. `Authority::MpidManager`
    1. if "push" model, notifying Client with `MpidHeader` when Client joins
    1. definition of `MPID_MESSAGE_TAG` and `MPID_HEADER_TAG`
    1. definition of `MpidMessage` and `MpidHeader`
    1. support Delete (for StructuredData only)
    1. address relocation (allows Client to use a fixed MPID address to connect to the network and allows "push" model - not required for "pull" model)

1. Client
    1. Put `MpidMessage`
    1. Get all `MpidHeader`s (pull model) or accept all/single `MpidHeader` (push model)
    1. Get `MpidMessage`.  This shall also include the work of removing corresponding `MpidHeader`s
    1. Delete `MpidMessage`
    1. address relocation (allows Client to use a fixed MPID address to connect to the network and allows "push" model - not required for "pull" model)
    1. address registration (not required if address relocation is used)

# Drawbacks

None identified, other than increased complexity of Vault and Client codebase.

# Alternatives

No other in-house alternatives have been documented as yet.

If we were to _not_ implement some form of secure messaging system, a third party would be likely to implement a similar system using the existing interface to the SAFE Network.  This would be unlikely to be as efficient as the system proposed here, since this will be "baked into" the Vault protocols.

We also have identified a need for some form of secure messaging in order to implement safecoin, so failure to implement this RFC would impact on that too.

# Unresolved Questions

1. Should we use the "push" or "pull" model?  At the moment, Qi and Fraser prefer the pull model on the basis that it involves less work: "address registration" or "address relocation" procedures are not required as the MPID Manager is always responding to requests from the MPID Client directly.  Furthermore, changing from the pull to push model at a later stage would appear to involve less wasted coding effort than vice versa.

1. `MpidMessage` and `MpidHeader` are wrapped in `StructuredData` instances in order to allow Delete requests on them and to allow them to be given different tags (otherwise they could be ImmutableData, since they don't mutate).  Should Routing, which already has to define these two types, be made "aware" of these two types?  (i.e. should they get added to the [`::routing::data::Data` enum][6])

    Identified pros:
    - less wrapping/parsing, so simpler for Vaults and Clients to deal with
    - more efficient (smaller messages to transmit and store)
    - no need for Routing to publicly expose `MPID_MESSAGE_TAG` or `MPID_HEADER_TAG`

    Identified cons:
    - increased complexity in Routing
    - would need to add the sender's public key to the header Put flow

    Qi prefers using `StructuredData`, Fraser prefers not using `StructuredData` unless the effort required by Routing to accommodate the new types is significant.

1. By having the `recipient` field inside `MpidMessage::signed_header`, a `sign::verify` call is required at every hop to parse the header and identify the recipient.  An alternative is to duplicate the `recipient` field in the `MpidMessage` itself.

    Identified pros of the alternative are:
    - avoids the computational cost of unnecessary signature verification
    - avoids the computational cost of parsing the header

    Identified cons of the alternative are:
    - increased message size (affects transmission and storage of the message)
    - requires a check at whatever stage the header is parsed that the duplicated `recipient` fields match

    So, should we use the proposed version of `MpidMessage` or should we add a `recipient` field?  Qi and Fraser prefer adding the field if benchmarking shows it's worthwhile

1. Should we give the MPID Client the ability to retrieve the full list of `MpidMessage`s or their corresponding `MpidHeader`s from its own network inbox?  This would take the form of a Get for `StructuredData` either with a new tag type specifically for this action or by targeting the Get at the Client's own ID rather than a specific Message ID.

    Identified pros:
    - useful to allow the Client to not have to guess which messages have been delivered

    Identified cons:
    - more coding required

    Qi and Fraser would prefer to add this.

1. Should we give the MPID Client the ability to retrieve a single `MpidHeader` from its own network inbox?  This would take the form of a Get for `StructuredData` with tag type `MPID_HEADER_TAG` (identical to a Client trying to retrieve a message which was sent to it except tag for that Get request is `MPID_MESSAGE_TAG`)

    Identified pros:
    - useful to allow the Client to not have to guess if a single message has been delivered

    Identified cons:
    - more coding required

    Qi and Fraser would prefer to add this.

1. Should we replace the `index`, `parent_index` and `sender` fields of `MpidHeader` with a required timestamp field?

    Identified pros:
    - it's user-friendly since it avoids the user having to encode it into the message contents or header `subject` field
    - almost all messaging systems have timestamps associated with the messages whereas not all systems have a need for threading (which the index fields allow) - we'd be more similar to norms if we did this
    - it would decrease the likelihood of a name collision of message headers
    - more efficient (smaller messages to transmit and store)

    Identified cons:
    - gives the wrong impression that the network will do something about the timestamp (e.g. drop old messages)

    Qi and Fraser prefer this.

# Future Work

It might be required to provide Vault-to-Vault, Client-to-Vault or Vault-to-Client communications in the future.  Potential use cases for this are:

1. Vault-to-Client notification of a successful safecoin mining attempt
1. Client-to-Vault request to take ownership of the Vault
1. Vault-to-Client notification of low resources on the Vault

In this case, the existing library infrastructure would probably need significant changes to allow a Vault to act as an MPID Client (e.g. the MPID struct is defined in the SAFE Client library).

Another point is that (as with MAID accounts), there is no cleanup done by the network of MpidMessages if the user decides to stop using SAFE.

Also, not exactly within the scope of this RFC, but related to it; MPID packets at the moment have no human-readable name.  It would be more user-friendly to provide this functionality.




[0]: https://github.com/maidsafe/safe_client/blob/c4dbca0e5ee8122a6a7e9441c4dcb65f9ef96b66/src/client/user_account.rs#L27-L29
[1]: #mpidheader
[2]: #mpidmessage
[3]: #outbox
[4]: #inbox
[5]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/structured_data.rs#L22-L34
[6]: https://github.com/maidsafe/routing/blob/7c59efe27148ea062c3bfdabbf3a5c108afc159c/src/data.rs#L24-L33
[7]: #messaging-format-among-nodes
