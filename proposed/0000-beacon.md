- Feature Name: beacon
- Type: enhancement
- Related components: crust
- Start Date: 12-11-2015
- RFC PR:
- Issue number:

# Summary

Create a library for beaconing (UDP broadcasting).

# Motivation

Crust needs a way to discover peers on the local network. As this is a common
need for many P2P applications it makes sense to split this functionality into
a separate library.

# Detailed design

The bulk of this work is already implemented. However this RFC proposes a few
changes to the existing implementation:
 * The API should be made more general (ie. not Crust-specific)
 * The API should use the interruptible-blocking-call style design of the other
   crust-refactor RFCs.
 * The implementation should allow multiple beacons in multiple processes to
   listen on the same port using. This will allow more than two crust instances
   on the one machine to discover each other.

The full API is given below:

```rust
/// A beacon used for broadcasting short messages to the local network and receiving them.
impl<'a> Beacon<'a> {
    /// Create a beacon that listens and broadcasts on `port`. The beacon starts a background thread
    /// that repeatedly sends `data` prefixed with `id_bytes` bytes of some randomly generated
    /// data. The message is resent every `period`.
    fn new<D>(port: u16, id_bytes: usize, data: D, period: Duration)
            -> (Beacon<'static>, BeaconController<'static>)
        where D: AsRef<[u8]> + Send + 'static

    /// This is identical to `new` except that it allows the beacon to send borrowed data so long
    /// as the data outlives the `Beacon`.
    fn new_scoped<'a, 'b: 'a, D>(port: u16,
                                 id_bytes: usize,
                                 data: D,
                                 period: Duration,
                                 scope: crossbeam::Scope<'a>)
            -> (Beacon<'a>, BeaconController<'a>)
        where D: AsRef<[u8]> + Send + 'b

    /// block until we receive a message from another beacon. This method will return any data that
    /// is received on the port where the first `id_bytes` of the packet does not equal that of the
    /// data we're sending.
    fn next_message(&mut self, buf: &mut Vec<u8>) -> Option<()>
}

impl<'a> BeaconController<'a> {
    /// Set the data being sent by the beacon.
    fn set_data<'b: 'a, D>(&mut self, data: D)
        where D: AsRef<[u8]> + Send + 'b

    /// Set the sending period of the beacon.
    fn set_period(&mut self, period: Duration)
}

impl<'a> Drop for BeaconController<'a> {
    // unblock the `next_message` call and destroy the `Beacon`
}
```

This is loosely based on the [zbeacon design doc](http://hintjens.com/blog:32).
One difference is that zbeacon will filter any received message that is
identical to the one it is sending in order to filter it's own echo. A problem
with this is that there's a race condition if the beacon sends a packet, has it's
data changed, and then receives it's echo. Instead, this beacon generates an
immutable `id_bytes` of data upon creation which it sends at the start of every
packet. These bytes are then used for filtering. This still gives the user full
control over the wire protocol as they can simply set `id_bytes` to zero and
implement their own filtering if they wish.

# Drawbacks

Creates more crates, arguably unnecessarily.

# Alternatives

Not do this.

# Unresolved questions

None.

