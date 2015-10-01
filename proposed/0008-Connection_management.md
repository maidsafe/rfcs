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

A first objective is the replacement of previous conditional variables with a clean flow through states.  This enables rebootstrapping, ie. when the connections to the network are lost, routing can automatically reconnect.  The states proposed in flow-order are `Disconnected`, `Bootstrapped`, `Relocated`, `Connected`, `GroupConnected`, `Terminated`.

A full node will start at `Disconnected` and follow the states in the above order until `GroupConnected`. If a node would get disconnected then the cycle can restart at `Disconnected`.  Only when the user calls `::stop()`, does routing enter `Terminated`.  At `Terminated` the node remains active but does not accept new messages and terminates his connections to the network.

A full node that cannot bootstrap (as such `Disconnected`), but accepts a connection from another node, will jump from `Disconnected` to `Relocated` by assigning itself a name.  Otherwise its behaviour is identical to normal behaviour.

A client will have a reduced cycle: `Disconnected`, `Bootstrapped`, `Terminated`.  Rebootstrapping is achieved through cycling through the first two states.

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

## On ConnectRequest and on ConnectResponse

```rust
fn handle_on_connect_request(&mut self, connect_request) {
    if self.core.check_routing_node(&connect_request.requester)
        // address relocation currently still works through the connection_cache
        // this should be checked here.
        // This cache can be removed after Address Relocation has been corrected
        && self.connection_cache.contains_key(&connect_request.requester) {
        self.service.connect(connect_request.external_endpoints);
        self.core.expect_connection(connect_request.clone());
        self.send_connect_response(connect_request);
    }
}

fn handle_on_connect_response(&mut self, connect_response) {
    if self.verify_response(&connect_response) {
        self.service.connect(connect_response.external_endpoints);
        self.core.expect_connection(connect_response.clone());
    }
}
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

## Hello and confirmation

```rust
fn handle_hello(&mut self, connection, ::direct_message::Hello) {
    match self.core.match_unknown_connection(connection) {
        Some(unknown_connection) => {
            if hello.contains_verified_connect_response() {
                match self.core.match_request_with_response(hello, connection) {
                    Some(confirmed_connection) => self.confirm_connection(confirmed_connection);
                    None => { self.service.drop_node(connection); return; }
                };
            } else {
                unknown_connection.insert_hello(hello);
            };
        },
        None => { self.service.drop_node(connection); return; },
    };
}

fn on_confirmation(&mut self, confirmation, connection) {
    // a confirmation must always be sent on the primary connection
    match self.core.match_unknown_connection(connection) {
        Some(unknown_connection) => {
            if unknown_connection.claimant == confirmation.claimant {
                match self.core.contains_expected_connection_response(
                    confirmation.claimant) {
                    Some(secondary_connection) => {
                        self.service.
                        [TO BE CONTINUED]
                    }
                }

            }
        }
    }
}
```

NOTE: this is unfinished and the implementation for a bootstrap connection, is not integrated in the above pseudo-code.

## Updates to Existing Code

1. Merge the `RoutingNode` functions `handle_new_connection` and `handle_new_bootstrap_connection`.
1. Remove `Unidentified` connections from `ConnectionName`.
1. In the event of disconnect implement re-bootstrapping.
1. Integrate `Hello` in line with crust events and updated methodology.

# Drawbacks

A double connection is established to ensure that a connection can be made in both direction. On cryptographically establishing the valid two-way-established connection, the original connection (from network to requester) is confirmed and the secondary connection is dropped.  This imposes more work to establishing a connection, but is fundamental to ensure the correct routing topology.  It is argued that without this secondary connection it is not possible to cryptographically ensure that IP-connection is authentic to the routing connection.

# Alternatives

What other designs have been considered? What is the impact of not doing this?

# Unresolved questions

What parts of the design are still to be done?
