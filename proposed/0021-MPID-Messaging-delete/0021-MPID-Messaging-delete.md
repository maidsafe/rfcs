- Feature Name: Deletion
- Type: Enhancement
- Related components: [mpid_messaging](https://github.com/maidsafe/mpid_messaging), [safe_vault](https://github.com/maidsafe/safe_vault), [safe_client](https://github.com/maidsafe/safe_client)
- Start Date: (fill me in with today's date, DD-MM-YYYY)
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

The `MpidMessageWrapper` enum used as the serialised `PlainData::value` for MPID-messaging `Put/Post` requests/responses should for the sake of consistency include `Delete` requests. 

# Motivation

The MPID-Messaging mechanism as it stands uses the `PlainData` type to issue requests and responses between peers. On receipt of any message a vault parses the outer message before passing it to the relevant persona. Currently `Put` and `Post` for a `PlainData` type are passed to the `MpidManager` to process by further parsing the `PlainData::value` and acting in accordance with resultant type. For consistency it is proposed to add to the `MpidMessageWrapper` values that can be parsed by the `MpidManager` on receipt of `Delete` requests arriving as `PlainData`.

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

No functionality is currently in place to handle the deletion.

# Unresolved questions

N/A
