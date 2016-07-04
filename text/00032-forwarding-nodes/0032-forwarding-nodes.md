- Feature Name: Forwarding nodes
- Status: proposed proposed
- Type: new feature
- Related components: kademlia_routing_table, routing
- Start Date: 09-05-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

A proposal to allow nodes to limit data sizes of messages forwarded through them.

# Motivation

In a decentralised network made up primarily of home PC's or smaller nodes there will be a
physical limit to upload capability. This is prevalent in many areas where asymmetric DSL
lines are used. The upload speeds in these cases are less than 1Mb/s. In such cases upload
(i.e. ability to forward) a message of 1Mb or more is prohibitive.

# Detailed design

In detail this proposal is very simple. Each node in a connect request will include a 2Mb
chunk of data if it can indeed act as a full node. Nodes that cannot pass a full 2Mb chunk can
instead pass None to the `Optional field` for connect payload. The routing table will then
flag these nodes as full or not wiht a simple `bool`.

On sending messages through th network these nodes are excluded from any path. Therefore the
routing table will require a flag to be passed to the `target nodws` function to state
whether we want only nodes capable of forwarding large data or not.


# Drawbacks

This does introduce an inbalance in sending network data through the DHT, but one which seems
very difficult to overcome without a system such as this.

# Alternatives

Nodes that cannot transmit such large payloads, queue the messages and drop them eventually.
This is wasteful and may introduce missing messages from the network

# Unresolved questions

To be added
