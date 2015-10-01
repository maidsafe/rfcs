- Feature Name: Improved Connection Management
- Type: Enhancement
- Related Components: routing, crust
- Start Date: 23-09-2015
- RFC PR: (leave this empty)
- Issue Number: (leave this empty)

# Summary

Introduce the notion of clearly defined connection phases and establish directed routing network connections in conjunction with crust version 0.3 and later. The combined objective with crust proposed here for routing is to prepare for NAT traversal using UDP/uTP.

# Motivation

Starting with crust version 0.3, the API provides directional information on newly established connections.  This information is helpful for establishing a routing network with the correct topology. Already at bootstrapping it simplifies the logic for the routing node. No artificial distinction is needed for a "new bootstrap connection", as that depends on the state of the node and whether the node accepts or has connected to another node.

# Detailed design

# Implementation blueprint

## State

Introduce a `State` object to `RoutingCore` representing the distinct states of connectedness for network entities, characterised as follows.

```rust
pub enum State {
    /// There are no connections.
    Disconnected,
    /// There are only bootstrap connections, and we do not yet have a name.
    Bootstrapped,
    /// There are only bootstrap connections, and we have received a name.
    Relocated,
    /// There are 0 < n < GROUP_SIZE routing connections, and we have a name.
    Connected,
    /// There are n >= GROUP_SIZE routing connections, and we have a name.
    GroupConnected,
    /// ::stop() has been called.
    Terminated,
}
```

## Expected Connections

The following types and events form an integral part of the connection management proposal.

```rust
pub struct crust::Connection {
    transport_protocol: Protocol,
    peer_addr: SocketAddrW,
    local_addr: SocketAddrW,
}

enum ExpectedConnection {
    InternalRequest::Connect(ConnectRequest),
    InternalResponse::Connect(ConnectResponse, SignedToken)
}

pub struct ExpectedConnections {
    lru_cache: LruCache<crust::Connection, ExpectedConnection>
}

::crust::Event::OnConnect(::crust::Connection)
::crust::Event::OnAccept(::crust::Connection)
::crust::Event::BootstrapFinished
::crust::Event::ExternalEndpoints(::crust::Endpoint)
```

## On accept and on connect

```rust
fn handle_on_accept(&mut self, connection) {
    match self.core.state() {
        State::Disconnected => {
            // on assigning a name, we become State::Relocated
            self.core.assign_name(self_relocated_name);
        },
        State::Bootstrapped => { self.service.drop_node(connection); return; },
        State::Relocated => {},
        State::Connected => {},
        State::GroupConnected => {},
        State::Terminated => { self.service.drop_node(connection); return; },
    };
    self.core.add_unknown_connection(connection);
}

fn handle_on_connect(&mut self, connection) {
    match self.core.state() {
        State::Disconnected => {
            // accept as bootstrap connection
            self.core.add_bootstrap_connection(connection.clone());
            self.send_hello(connection);
            return;
        },
        State::Bootstrapped => { self.service.drop_node(connection); return;
            /* refuse connection, only have one bootstrap connection */ },
        State::Relocated => {},
        State::Connected => {},
        State::GroupConnection => {},
        State::Terminated => { self.service.drop_node(connection); return; },
    };
    match self.core.match_expected_connection(connection) {
        Some(ref mut expected_connection) => {
            expected_connection.insert_connection(connection.clone());
            self.send_hello(connection);
        },
        None => { self.service.drop_node(connection); return; },
    };
}
```

## Hello

```rust
fn handle_hello(&mut self, connection, ::direct_message::Hello) {

}
```

## Updates to Existing Code

1. Merge the `RoutingNode` functions `handle_new_connection` and `handle_new_bootstrap_connection`.
1. Remove `Unidentified` connections from `ConnectionName`.
1. In the event of disconnect implement re-bootstrapping.
1. Integrate `Hello` in line with crust events and updated methodology.

# Drawbacks

N/A

# Alternatives

What other designs have been considered? What is the impact of not doing this?

# Unresolved questions

What parts of the design are still to be done?
