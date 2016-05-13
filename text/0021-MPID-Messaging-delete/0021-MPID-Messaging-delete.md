- Feature Name: Add deletion values to `MpidMessageWrapper`
- Status: active
- Type: Enhancement
- Related components: [mpid_messaging](https://github.com/maidsafe/mpid_messaging), [safe_vault](https://github.com/maidsafe/safe_vault), [safe_client](https://github.com/maidsafe/safe_client)
- Start Date: 03-02-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/95
- Supersedes:
- Superseded by:

# Summary

The `MpidMessageWrapper` enum used as the serialised `PlainData::value` for MPID-messaging `Put/Post` requests/responses should for the sake of consistency as well as efficiency include `Delete` requests. Currently `Put/Post` requests/responses are passed to the `MpidManager` to handle, whereas, `Delete` requests for messages in a `Client`'s `Outbox` or headers in the `Inbox` are not catered for. Adding `DeleteMessage` and `DeleteHeader` to `MpidMessageWrapper` will allow those messages to be passed to the `MpidManager` in order to be handled properly based on information only available to the `MpidManager` with respect to messaging.  

# Motivation

The MPID-Messaging mechanism as it stands uses the `PlainData` type to issue requests and responses between peers. On receipt of any message a vault parses the outer message before passing it to the relevant persona. Currently `Put` and `Post` for a `PlainData` type are passed to the `MpidManager` to process by further parsing the `PlainData::value` and acting in accordance with resultant type. It is proposed to add to the `MpidMessageWrapper` values that can be parsed by the `MpidManager` on receipt of `Delete` requests arriving as `PlainData`.

# Detailed design

The current `MpidMessageWrapper` is given as,

```
pub enum MpidMessageWrapper {
    Online,
    PutMessage(MpidMessage),
    PutHeader(MpidHeader),
    GetMessage(MpidHeader),
    OutboxHas(Vec<XorName>),
    OutboxHasResponse(Vec<MpidHeader>),
    GetOutboxHeaders,
    GetOutboxHeadersResponse(Vec<MpidHeader>)
}
```

This proposal would involve changing that to,

```
pub enum MpidMessageWrapper {
    Online,
    PutMessage(MpidMessage),
    PutHeader(MpidHeader),
    GetMessage(MpidHeader),
    OutboxHas(Vec<XorName>),
    OutboxHasResponse(Vec<MpidHeader>),
    GetOutboxHeaders,
    GetOutboxHeadersResponse(Vec<MpidHeader>),
    DeleteMessage(XorName),
    DeleteHeader(XorName)
}
```

The effect of this on the `MpidManager` would be the inclusion of the following function,

```
pub fn handle_delete(&mut self, routing_node: &RoutingNode, request: &RequestMessage) -> Result<(), InternalError> {
    match Parse request content
        MpidMessageWrapper::DeleteMessage(name) => {
        	Delete a message from a client's outbox.
        }
        MpidMessageWrapper::DeleteHeader(name) => {
            Delete a header from a client's inbox.
       	}
    }
}
```

# Drawbacks

Since no `Authority` is defined for `MpidManager`'s, `PlainData` messages used for Mpid-Messaging arrive at a vault from `Client` to `ClientManager` preventing the use of `PlainData` in other scenarios.

# Alternatives

Add an `Mpidmanager` authority to separate the responsibilty of handling individual data types on a per persona basis. Mostly related to the drawback observed above, and will nevertheless require some form of deletion capability to be included, anticipated to be that suggested by the current proposal.

# Unresolved questions

N/A
