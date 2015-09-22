MPID Messaging system
=========

Design document 1.0

Introduction
============

In MaidSafe the ability for secured messaging is obvious and may take many forms, mail like, IM like etc. This document outlines the system componenets and design for general communications infrastructure and security.

Motivation
==========

The motivation for a messaging system is an obvious one, but a wider motivation is, as always in MaidSafe, a messaging system that is secure and private without the possibility of snooping tracking or being abused in any way. A large abuse of modern day digital communications is the ability for immoral entities such as spammers and the less than ethical marketing companies to flood the Internet with levels of unsolicited email to a level of circa 90%. This is a waste of good bandwidth and also a nuisance of significant proportions.

A MaidSafe messaging system will attempt to eradicate unwanted and intrusive communications. This would seem counter intuative to a network that purports to be more efficient, cheaper and faster than today's mechanisms. The flip side of this is the network's ability to work autonomously and follow pre-programmed rules. With such rules then the network can take care of people's inboxes and outboxes for incoming and outgoing mail respectively.

This design outlines a mechanism where the cost of messaging is with the sender as this would seem more natural. To achieve this, the sender will maintain messages in the network outbox until they are retrived by the recipient. If the email is unwanted the recipient simply does not retrieve the message. The sender will quickly fill their own outbox with undelivered mail and be forced to clean this up, themselves.

This paradigm shift will mean that the obligation to un-subscribe from mailing lists etc. is now with the owner of these lists. If people are not picking up mail, it is because they do not want it. So the sender has to do a better job. It is assumed this shift of responsibilities will lead to a better managed bandwidth solution and considerably less stress on the network and the users of the network.

Overview
========

A good way to look at the solution is that, rather than charging for unwanted mail with cash, the network charges with a limited resource and then prevents further abuse. In another apsect this is regulation of entities by the users of the system affected by that entity. Rather than build a ranking system to prevent bad behaviour, this proposal is actually the affected people acting independently. This protects the minorites who may suffer from system wide rules laid down by any designer of such rules.

Network OutBox
--------------

This is a simple data structure for now and will be a ```std::map``` ordered by the hash of the serialised and encrypted ```MpidMessage```  and with a user defined object to represent the message (value). The map will be named with the ID of the MPID it represents (owner). The data structure for the value will be

```c++
struct MpidMessage {
  PublicMpid::Name sender;
  PublicMpid::Name recipient;
  BoundedString<0, MAX_HEADER_SIZE> message_head;
  BoundedString<0, MAX_BODY_SIZE> message_body;
  Identity message_id, parent_id;
};

```

It needs to be highlighted that each above MpidMessage only targets one recipient. When a sender sending a message to multiple recipients, multiple MpidMessages will be created in the ```OutBox``` . This is to ensure spammers will run out of limited resource quickly, so the network doesn't have to suffer from abused usage.

Network Inbox
-------------

The network inbox is an even simpler structure and will be again named with the MpidName of the owner. This can be represented via a ```std::vector<MpidAlert>```

```c++
struct MpidAlert {
  Identity alert_id;
  Identity message_id, parent_id;
  PublicMpid::Name sender;
  BoundedString<0, MAX_HEADER_SIZE> message_head;
};
```

Messaging Format among nodes
--------------
The above defined ourbox/inbox and MpidMessage/MpidAlert structs are to be used internally in MpidManager and client.
The messaging format being used between client to network and among MpmidManagers is utilising structured data :
StructuredData (type_tag = 51000, identity = mpid_message.message_id, version = 0,
                data = mpid_message.serialised(),
                current_owner_keys = vec![mpid_message.sender],
                previous_owner_keys = vec![], singing_key = None)
StructuredData (type_tag = 51001, identity = mpid_alert.alert_id, version = 0,
                data = mpid_alert.serialised(),
                current_owner_keys = vec![mpid_alert.sender],
                previous_owner_keys = vec![], singing_key = None)


Message Flow
------------
```
        MpidManagers (A)                           MpidManagers (B)
           /  *                                    * \
Mpid (A) -> - *                                    * - <-Mpid (B)
           \  *                                    * /

```
1. The user at Mpid(A) sends MpidMessage to MpidManager(A) signed with the recipient included, with a Put request.
2. The MpidManagers(A) stores this message and perform the action() which sends the mpid_alert to MpidManagers(B) [the ```MpidAlert::alert_id``` at this stage is hash of the MpidMessage.
3. MpidManager(B) stores the mpid_alert and forwards it to Mpid(B) as soon as the client found online.
4. On receving the alert, Mpid(B) sends a ```retrieve_message``` to MpidManagers(B) via a Get request, which will be forwarded to MpidManagers(A).
5. MpidManagers(A) sends the message to MpidManagers(B) which is forwarded to MPid(B) if MPid(B) is online.
6. On receiving the message, Mpid(B) sends a remove request to MpidManagers(B) via Delete, MpidManagers(B) remove the corresponding alert and forward the remove request to MpidManager(A). MpidManagers(A) then remove the corresponding entry.
7. When Mpid(A) decides to remove the MpidMessage from the OutBox, if the message hasn't been retrived by Mpid(B) yet. The MpidManagers(A) group should not only remove the correspondent MpidMessage from their OutBox of Mpid(A), but also send a notification to the group of MpidManagers(B) so they can remove the correspodent MpidAlert from their InBox of Mpid(B).

_MPid(A)_ =>> |__MPidManager(A)__ (Put)(Alert.So) *->> | __MPidManager(B)__  (Store(Alert))(Online(Mpid(B)) ? Alert.So : (WaitForOnlineB)(Alert.So)) *-> | _Mpid(B)_ So.Retreive ->> | __MpidManager(B)__ *-> | __MpidManager(A)__ So.Message *->> | __MpidManager(B)__ Online(Mpid(B)) ? Message.So *-> | _Mpid(B)_ Remove.So ->> | __MpidManager(B)__ {Remove(Alert), Remove.So} *->> | __MpidManager(A)__ Remove

MPID Messaging Client
--------------
The messaging client, as described as Mpid(X) in the above section, can be named as nfs_mpid_client. It shall provide following key functionalities :

1. Send Message (Put from sender)
2. Retrieve Full Message (Get from receiver)
3. Remove sent Message (Delete from sender)
4. Accept Message Alert (when ```PUSH``` model used) and/or Retrive Message Alert (when ```PULL``` model used)

When ```PUSH``` model is used, nfs_mpid_client is expected to have it's own routing object (not sharing with maid_nfs). So it can connect to network directly allowing the MpidManagers around it to tell the connection status directly.

Such seperate routing object is not required when ```PULL``` model is used. It may also have the benefit of saving the battery life on mobile device as the client app doesn't need to keeps nfs_mpid_client running all the time.

Planned Work
============
1, Vault
    a, MpidManager::OutBox
    b, MpidManager::InBox
    c, detecting of client, when PUSH model used
    d, sending message flow
    e, retrieving message flow
    f, deleting message flow

2, Routing
    a, Authority::MpidManager
    b, PUSH model
        Notifying client with mpid_alert when client join
        This is not required if PULL model is used

3, Client
    a, Send Message
    b, Get Message (or Accept Message)
        This shall also includes the work of removing correspondent mpid_alerts
    c, Delete Message

