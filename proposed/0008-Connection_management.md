- Feature Name: Improved Connection Management
- Type: Enhancement
- Related Components: routing, crust
- Start Date: 23-09-2015
- RFC PR: (leave this empty)
- Issue Number: (leave this empty)

# Summary

Introduce the notion of clearly defined connection phases and establish directed routing network connections in conjunction with crust.

# Motivation

Both outgoing and incoming connections are created to peers with unknown direction. Use the connect/accept connection code available from crust to store timed state in-line with crust connection handling changes for connections in either direction. This will allow us to determine unambiguously who the connection initiator is and act accordingly.

# Detailed design

## State

Introduce a `State` object to `RoutingCore` representing the distinct phases of execution for network entities characterised as follows.

1. Initially client/node in disconnected state, and when all connections are dropped/lost.
1. Bootstrapping, initiated in constructor by calling crust service function and halted by crust event sent over channel.
1. If non-client node, connected phase adds connections to routing table.
1. In order to prevent any further network activity a terminated state.

```rust
pub enum State {
    Disconnected,
    Bootstrapping,
    Connected,
    Terminated,
}
```

## Expected Connections

For connections, create and add to utils folder a timed `ExpectedConnections` object for key type `crust::Connection`, and value, new type, `ExpectedConnection`. An object of type `ExpectedConnections` replaces the current `connection_filter` in `RoutingNode`.

```rust
pub struct Connection {
    transport_protocol: Protocol,
    peer_addr: SocketAddrW,
    local_addr: SocketAddrW,
}

enum ExpectedConnection {
    InternalRequest::Connect(ConnectRequest),
    InternalResponse::Connect(ConnectResponse, SignedToken)
}

pub struct ExpectedConnections {
    lru_cache: LruCache<Connection, ExpectedConnection>
}
```

For incoming connect requests, we want to handle, store the `ExpectedConnection::ConnectRequest(ConnectRequest)` in the timed `ExpectedConnections` then set-up and try to rendezvous connect to the peer. For incoming connect responses check the returned `ConnectRequest` was sent by us and store the `ExpectedConnection::ConnectResponse(ConnectResponse)` in the timed `ExpectedConnections` then set-up and try to rendezvous connect to peer. The rendezvous connection set-up and establishment will be initiated by routing and handled by crust, successful completion resulting in receipt of a crust OnConnect/OnAccept event within the time limit for the stored expected `ConnectRequest/ConnectResponse`. The new connection details can then be added to the routing table and removed from `ExpectedConnections`.
 

## Updates to Existing Code

1. Merge the `RoutingNode` functions `handle_new_connection` and `handle_new_bootstrap_connection`.
1. Remove `Unidentified` connections from `ConnectionName`.
1. In the event of disconnect implement re-bootstrapping.
1. Update Hello in line with crust events.

# Drawbacks

N/A

# Alternatives

As an enhancement to current design, over crust network protocol handling, when hole-punching is complete the proposed design will explicitly cater for it.

# Unresolved questions

Testing should clear up any collaborative issues between routing and crust that may arise.
