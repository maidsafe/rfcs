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

A fundamental principle is that the cost of messaging is with the sender, primarily to deter unsolicited messages.  To achieve this, the sender will maintain messages in a network outbox until they are retrived by the recipient.  If the message is unwanted, the recipient simply does not retrieve the message.  The sender will thus quickly fill their own outbox with undelivered mail and be forced to clean this up, themselves before being able to send further messages.

This paradigm shift will mean that the obligation to unsubscribe from mailing lists, etc. is now with the owner of these lists. If people are not picking up mail, it is because they do not want it.  So the sender has to do a better job.  It is assumed this shift of responsibilities will lead to a better managed bandwidth solution and considerably less stress on the network and the users of the network.

## Details of Structs and Consts

The two relevant structs (other than the MPID itself which is a [standard Client key][0]) are the [`MpidHeader`][1] and the [`MpidMessage`][2].

Broadly speaking, the [`MpidHeader`][1] contains metadata and the [`MpidMessage`][2] contains a signed header and message.  We also want to define some upper limits which will be described later.

### Consts

```rust
pub const MPID_MESSAGE_TAG: u64 = 51000;
pub const MPID_HEADER_TAG: u64 = 51001;
pub const MAX_SUBJECT_SIZE: usize = 128;  // bytes
pub const MAX_BOX_SIZE: usize = 1 << 27;  // bytes, i.e. 128 MiB, max total of outbox AND inbox
```

### `MpidHeader`

```rust
pub struct MpidHeader {
    pub sender: ::routing::NameType,
    pub recipient: ::routing::NameType,
    pub index: u32,
    pub parent_index: Option<u32>,
    pub subject: Vec<u8>,
    pub fn name(self) -> { hash(self.serialised()) }
}
```

The `sender` and `recipient` fields are hopefully self-explanatory.  The `index` and `parent_index` are intended to allow threading or sequencing of messages, while `subject` allows passing arbitrary data (e.g. a subject line).

The `subject` field must not exceed `MAX_SUBJECT_SIZE` bytes.

### `MpidMessage`

```rust
pub struct MpidMessage {
    pub serialised_header: Vec<u8>,
    pub signed_body: Vec<u8>,
    pub fn name(self) -> { hash(self.serialised()) }
    pub fn header_name(self) -> { hash(self.serialised_header) }
}
```
Each `MpidMessage` instance only targets one recipient.  For multiple recipients, multiple `MpidMessage`s need to be created in the [Outbox][3] (see below).  This is to ensure spammers will run out of limited resources quickly.

## Outbox

This is a simple data structure for now and will be a hash map of serialised and encrypted `MpidMessage`s.  There will be one such map per MPID (owner).

## Inbox

This is an even simpler structure and again there will be one per MPID (owner).  This can be implemented as a `Vec<MpidHeader>`.

## Messaging Format Among Nodes

The above defined outbox/inbox and MpidMessage/MpidHeader structs are to be used internally in MpidManager and client.

The messaging format being used between client to network and among MpmidManagers is utilising structured data :
```rust
let sd_for_mpid_message = StructuredData {
    type_tag : MPID_MESSAGE_TAG,
    identity : mpid_message.name(),
    previous_owner_keys = vec![],
    version : 0,
    data : mpid_message.serialised(),
    current_owner_keys : vec![mpid_message.sender],    
    previous_owner_signatures : vec![]
}
```
```rust
let sd_for_mpid_header = StructuredData {
    type_tag : MPID_HEADER_TAG,
    identity : mpid_message.header_name(), // or mpid_header.name()
    previous_owner_keys : vec![],
    version : 0,
    data : mpid_message.serialised_header, // or mpid_header.serialised()
    current_owner_keys : vec![mpid_message.sender], // or vec![mpid_header.sender] 
    previous_owner_signatures : vec![]
}
```

## Message Flow

```
        MpidManagers (A)                           MpidManagers (B)
           /  *                                    * \
Mpid (A) -> - *                                    * - <-Mpid (B)
           \  *                                    * /

```
1. The user at Mpid(A) sends MpidMessage to MpidManager(A) signed with the recipient included, with a Put request.
2. The MpidManagers(A) stores this message and perform the action() which sends the mpid_header to MpidManagers(B).
3. MpidManager(B) stores the mpid_header and forwards it to Mpid(B) as soon as the client found online.
4. On receving the notification, Mpid(B) sends a ```retrieve_message``` to MpidManagers(B) via a Get request, which will be forwarded to MpidManagers(A).
5. MpidManagers(A) sends the message to MpidManagers(B) which is forwarded to MPid(B) if MPid(B) is online.
6. On receiving the message, Mpid(B) sends a remove request to MpidManagers(B) via Delete, MpidManagers(B) remove the corresponding header and forward the remove request to MpidManager(A). MpidManagers(A) then remove the corresponding entry.
7. When Mpid(A) decides to remove the MpidMessage from the OutBox, if the message hasn't been retrived by Mpid(B) yet. The MpidManagers(A) group should not only remove the correspondent MpidMessage from their OutBox of Mpid(A), but also send a notification to the group of MpidManagers(B) so they can remove the correspodent mpid_header from their InBox of Mpid(B).

_MPid(A)_ =>> |__MPidManager(A)__ (Put)(Header.So) *->> | __MPidManager(B)__  (Store(Header))(Online(Mpid(B)) ? Header.So : (WaitForOnlineB)(header.So)) *-> | _Mpid(B)_ So.Retreive ->> | __MpidManager(B)__ *-> | __MpidManager(A)__ So.Message *->> | __MpidManager(B)__ Online(Mpid(B)) ? Message.So *-> | _Mpid(B)_ Remove.So ->> | __MpidManager(B)__ {Remove(Header), Remove.So} *->> | __MpidManager(A)__ Remove

## MPID Messaging Client

The messaging client, as described as Mpid(X) in the above section, can be named as nfs_mpid_client. It shall provide following key functionalities :

1. Send Message (Put from sender)
2. Retrieve Full Message (Get from receiver)
3. Remove sent Message (Delete from sender)
4. Accept Message header (when ```PUSH``` model used) and/or Retrive Message Alert (when ```PULL``` model used)

When ```PUSH``` model is used, nfs_mpid_client is expected to have it's own routing object (not sharing with maid_nfs). So it can connect to network directly allowing the MpidManagers around it to tell the connection status directly.

Such seperate routing object is not required when ```PULL``` model is used. It may also have the benefit of saving the battery life on mobile device as the client app doesn't need to keeps nfs_mpid_client running all the time.

## Planned Work

1, Vault
'''
    a, MpidManager::OutBox
    b, MpidManager::InBox
    c, detecting of client, when PUSH model used
    d, sending message flow
    e, retrieving message flow
    f, deleting message flow
    g, churn handling and refreshing for account_transfer
    h, mpid_client addressing (if mpid address registratioin procedure to be undertaken)
'''
2, Routing
'''
    a, Authority::MpidManager
    b, PUSH model
        Notifying client with mpid_alert when client join
        This is not required if PULL model is used
    c, Definition of MPID_MESSAGE_TAG and MPID_HEADER_TAG
    d, Definition of MpidMessage and MpidHeader
    e, Support Delete (for StructuredData only)
    f, address relocation (if allows client using fixed mpid address connecting network)
        not required if 1.h is implemented
'''
3, Client
'''
    a, Send Message
    b, Get Message (or Accept Message)
        This shall also includes the work of removing correspondent mpid_alerts
    c, Delete Message
    d, address relocation (if allows client using fixed mpid address connecting network)
        not required if 1.h is implemented
'''

# Drawbacks

None identified, other than increased complexity of Vault and Client codebase.

# Alternatives

No other in-house alternatives have been documented as yet.

If we were to _not_ implement some form of secure messaging system, a third party would be likely to implement a similar system using the existing interface to the SAFE Network.  This would be unlikely to be as efficient as the system proposed here, since this will be "baked into" the Vault protocols.

We also have identified a need for some form of secure messaging in order to implement safecoin, so failure to implement this RFC would impact on that too.

# Unresolved questions



# Future Work

It might be required to provide Vault-to-Vault, Client-to-Vault ot Vault-to-Client communications in the future.  Potential use cases for this are:

1. Vault-to-Client notification of a successful safecoin mining attempt
1. Client-to-Vault request to take ownership of the Vault
1. Vault-to-Client alert of low resources on the Vault

In this case, the existing library infrastructure would probably need significant changes to allow a Vault to act as an MPID Client (e.g. the MPID struct is defined in the SAFE Client library).





[0]: https://github.com/maidsafe/safe_client/blob/c4dbca0e5ee8122a6a7e9441c4dcb65f9ef96b66/src/client/user_account.rs#L27-L29
[1]: #mpidheader
[2]: #mpidmessage
[3]: #outbox
[4]: #inbox
