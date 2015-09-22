- Feature Name: Improved connection management.
- Type enhancement
- Related components routing, crust
- Start Date: (fill me in with today's date, DD-MM-YYYY)
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Establish directed routing network connections in conjunction with crust.

# Motivation

The notion of a clearly defined connection phase is required. Also, both outgoing and incoming connections are created to peers with unknown direction. The code should be updated to store timed 'State' in-line with crust connection handling changes for connections in either direction. This will allow us to determine unambiguously who the connection initiator is and act accordingly.

# Detailed design

Bootstrap connection handling can be seen as a separate stage of network interaction from active/passive connection handling. Crust has put in place the separation by providing a start and end of bootstrapping phase over the crust event channel. We therefore have,

1. Remove bootstrap handling specific functions, completely if logic permits, to favour connection specific functions in routing. For this we'll require a State object in routing_core along the, preliminary, lines of,

pub enum Phase {
    Disconnected(bool),
    Bootstrapping(bool),
    Connected(bool),
    Terminated(bool),
}

pub struct State {
    phase: Phase
}

The routing_node fn's handle_new_connection and handle_new_bootstrap_connection can presumably be merged in the process.

2. For active/passive connections we require a new type,

enum ExpectedConnection {
    ConnectRequest(ConnectRequest),
    ConnectResponse(ConnectResponse)
}

3. For active/passive connections a timed filter object is required for key type 'crust::Connection', and value ExpectedConnection.

4. In the event of disconnect allow re-boostrapping.

# Drawbacks

Why should we *not* do this?

# Alternatives

What other designs have been considered? What is the impact of not doing this?

# Unresolved questions

What parts of the design are still to be done?
