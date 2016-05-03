- Feature Name: Crust mock
- Status: implemented
- Type: feature
- Related components: crust, routing
- Start Date: 18-02-2016
- RFC PR:
- Issue number:

# Summary

Drop-in mock replacement for Crust to make it easier to unit test libraries and applications that use Crust.

# Motivation

Testing Crust-depending libraries on real network (even on LAN or localhost) is cumbersome, slow and potentially expensive. A drop in mock replacement for Crust could make these test simpler and faster, make it easier to diagnose failures, and help cover edge cases difficult to set up on real network.

# Detailed design

This section details one possible implementation of the Crust Mock.

## Changes to routing

Add a feature gate which when enabled will replace `crust::Service` with the mock version. There might be need to replace other types with mock version. One potential candidate is `crust::OutConnectionInfo`.

## Features of `crust::mock::Service`

- Do not hit real network (not even LAN or localhost), do not use TCP/UDP sockets.
- Simulate all the relevant config files crust uses (crust config, bootstrap cache, etc...). Possibly by accepting structs, representing those configs, as parameters to the constructor.
- Pass simulated packets (messages) between different instances of the mock service.
- Enable, disable, pause and resume incoming and outgoing traffic for each `mock::Service` individually (and/or for the whole mock network), to simulate various network failures.
- Full control over the order of the simulated network operations. The whole system SHOULD be synchronous.
- OPTIONALLY: intercept and examine messages sent over the simulated network
- OPTIONALLY: ability to set expectations of various network events and verify that they really occurred.

## Implementation details

Refer to the `implementation-sketch.rs` file in the same directory as this document.

## Additional notes

Although this should eventually become part of crust crate, it might make things easier to initially start developing it as part of routing. Then, routing unit tests utilizing it could be written in parallel, serving double role as tests for routing and sanity checks for the mock service. After it sufficiently stabilizes, it can be extracted and incorporated into crust.

# Drawbacks

TODO (Why should we *not* do this?)

# Alternatives

## Do not mock crust, mock the lower layers (TCP/UDP protocol) instead.

Advantage of this is that there might be existing solution for this (TODO: investigate them). Disadvantage is that it might be too low-level for out purposes. For example, we would have to deal with things like hole-punching which are not interesting for the upper layers. On the other hand, it would be possible to use this to unit-test crust itself (but that is out of scope of this RFC).

## Testing on real network

Closest to actual usage, but very slow, unpredictable, potentially expensive and cumbersome to diagnose.

## Testing on local network

Faster than on real network, but getting sufficient number of physical machines might be expensive. Difficult to test connection failures and other error situations (as LANs tend to be more reliable than real networks). Still slow and cumbersome to diagnose.

## Testing on localhost

Tricky to test large networks as they tent to exhaust hardware resources (CPU, RAM), making testing slow and unpredictable. Diagnosis is still slow and cumbersome.

## Conditional compilation instead of trait

This is an alternative to defining the `ConnectionService` trait with two implementation (real and mock). Instead, the `#[cfg(test)]` attribute would be used to control whether to use the real `crust::Service` or the mocked one. We would still need the ability to pass a service-returning lambda to `routing::Core` to be able to pass fake config files, etc... to the mock service. That could be test only too.


# Unresolved questions

None so far.
