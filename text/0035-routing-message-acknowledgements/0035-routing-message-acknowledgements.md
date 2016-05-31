# Routing Message Acknowledgements

- Status: implemented
- Type: enhancement
- Related components: Routing
- Start Date: 11-05-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/133
- Supersedes:
- Superseded by:

# Summary

This RFC proposes that Routing's parallel message sending mechanism is replaced by a message
acknowledgement (ack) mechanism.

# Motivation

Routing currently implements sending messages via eight parallel routes for reliability and security
reasons.  While the routes for a given message will likely converge as the message nears its
destination, this is still a high cost to pay in terms of network traffic and processing effort by
Routing nodes.

The proposed ack mechanism should allow these costs to be reduced significantly.

# Detailed design

Rather than a message being sent via parallel routes, instead it will be sent via a single route and
will require an ack to be sent by the recipient(s).  Should an ack *not* be received within a
specified time, the message will be resent and the sender will begin waiting for this to be
acknowledged.  This will be repeated until the message is eventually acknowledged or until
`GROUP_SIZE` attempts have been made.

To avoid re-sending repeatedly via the same route, the `HopMessage` will have a new `route: u8`
field added.  At each hop, this will be used to choose which of the potential target nodes to choose
as the next recipient.

The ack value will be the SipHash of the `SignedMessage`.  The node will hold a `HashMap` of
unacknowledged messages with the key being the ack value:

```rust
struct UnacknowledgedMessage {
    msg: SignedMessage,
    route: u8,
    timer_token: u64,
}

pub struct Core {
    ...
    pending_acks: HashMap<u64, UnacknowledgedMessage>,
    ...
}
```

When an ack is received, the corresponding entry is removed from `pending_acks`.

If an entry times out and the `route` is `GROUP_SIZE - 1`, the entry is removed and no further
action is taken.  Otherwise, for timed-out entries the `route` is incremented by one, the
`timer_token` is updated with the result of a newly-scheduled `timer` event, and the `msg` is resent
using the updated `route` value as the corresponding value in the `HopMessage`.

From the recipient's perspective, an ack will be sent in response to every received message with the
exception of ack messages themselves.  The ack should be sent as early as possible, i.e. before the
message has been fully handled.

The ack message will take the form of a new variant of Routing's `ResponseContent`:

```rust
pub enum ResponseContent {
    ...
    Ack(u64),
    ...
}
```

As above, the value will be the SipHash of the received `SignedMessage`.

# Drawbacks

This is a less secure mechanism.  However, the security concerns will be addressed in a future RFC.

# Alternatives

There are some possible improvements which are not being considered at the moment in the interests
of simplicity.  Some ideas which have surfaced are:
* variable timeouts depending on the distance the message has to travel and/or the size of message
* selective acks - not all messages need acknowledged

# Unresolved questions

None.
