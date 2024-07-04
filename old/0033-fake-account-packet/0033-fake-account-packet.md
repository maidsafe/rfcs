# Fake Account Packet

- Status: proposed
- Type: enhancement
- Related components: SAFE Vault and Routing
- Start Date: 10-05-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/134
- Supersedes:
- Superseded by:

## Summary

This proposes an enhancement whereby the network will respond to a `Get` request for a non-existent
account packet (`StructuredData` with `type_tag` 0) with a fake account packet.

## Motivation

The objective is to deter an attack where random account packets are requested by a malicious Client
in the hopes of finding one in order to attempt decryption of it.

The defence is to have the network respond to requests for non-existent account packets with fakes
which will appear valid to the recipient.  This will make it expensive for the attacker, since it
won't know whether the decrypted packet will yield useful data or not.

## Detailed design

### Issues to Overcome

The main issue is to be able to have a group of Vaults agree a fake packet which will be accumulated
by the attacker - i.e. the group all need to generate an identical fake.  Essentially we just need a
single value agreed across the group which can then be used to seed a pseudorandom number generator.

The Vaults could all exchange a message in order to agree a seed, however this is undesirable since
it could easily be exploited as an amplification attack.

This leaves little scope for agreeing a single seed; the only piece of information which all the
Vaults know is the list of Vault names comprising the close group.  This is useless as a seed since
the attacker could simply monitor the individual Routing responses arriving and hence accrue this
list of Vault names.  It would then be easy to deduce that the responses contain a fake.

A more trivial issue is that while account packets are currently the highest-value targets for an
attacker, it should be possible to extend the defence to other `StructuredData` types easily.

### Proposed Solution

There is an example implementation at https://gitlab.com/Fraser999/Fake-Account-Packet.

The implementation here requires Vaults to hold a new piece of data; a [`SessionId`]
(https://gitlab.com/Fraser999/Fake-Account-Packet/blob/master/src/main.rs#L21) defined as

```rust
struct SessionId(pub [u8; SEEDBYTES]);
```

where `SEEDBYTES` is 32; the size of the sodiumoxide `Seed`.

This `SessionId` is created randomly by the Vault on startup and is exchanged with any peers with
which it establishes a connection.  Critically, this `SessionId` is never sent to any node (Vault or
Client) until the peer is being added to the Routing Table.  This should reduce the ability for an
attacker to harvest `SessionId`s.

This new piece of information per Vault (unknown by Clients) can now be used to generate a seed for
any given close group.  To avoid generating the same seed for all addressable elements within a
narrow address space it would be best to also mix in the requested data name.

In [this example](https://gitlab.com/Fraser999/Fake-Account-Packet/blob/master/src/main.rs#L33-53)
it is implemented by iterating through the data name and all peer `SessionId`s, taking the first
`u8` from each, then the second, and so on until we have built up a 32-byte array of `u8`s:

```rust
fn get_seed(id: &XorName, session_ids: &[SessionId]) -> Seed {
    let mut seed = Seed([0; SEEDBYTES]);

    let mut iters = vec![id.0.iter()];
    for session_id in session_ids {
        iters.push(session_id.0.iter());
    }

    let mut seed_index = 0;
    loop {
        for iter in &mut iters {
            seed.0[seed_index] = *iter.next().expect("Can't fail.");
            seed_index += 1;
            if seed_index == SEEDBYTES {
                return seed;
            }
        }
    }
}
```

To reduce the chances of disagreement, this function could use only the closest few peers rather
than the entire group for example.

Having agreed a seed across the group, it is now trivial for each peer to generate an identical
`StructuredData` packet.

In order to maintain separation of concerns and to increase extensibility, this can been provided
via a new trait; [`Fake`]
(https://gitlab.com/Fraser999/Fake-Account-Packet/blob/master/src/fake.rs):

```rust
pub trait Fake {
    fn fake(seed: &Seed) -> Self;

    fn fake_version(seed: &Seed) -> u64;

    fn to_rng_seed(seed: &Seed) -> Vec<u32> {
        let mut new_seed = vec![];
        let mut new_seed_elt = 0;
        let mut count = 0;
        for elt in &seed.0 {
            new_seed_elt += (*elt as u32) << count;
            count += 8;
            if count == 32 {
                new_seed.push(new_seed_elt);
                count = 0;
                new_seed_elt = 0;
            }
        }
        if count != 32 {
            new_seed.push(new_seed_elt);
        }
        new_seed
    }
}
```

This trait allows data types to implement their fake constructors as they see fit.  It also provides
a helper function to allow easy conversion from a `sodiumoxide::crypto::sign::Seed` to a `Vec<u32>`
which can be used by Rust random number generators.

An [`ExampleAccount`]
(https://gitlab.com/Fraser999/Fake-Account-Packet/blob/master/src/example_account.rs) which
implements the `Fake` trait is shown here:

```rust
#[derive(Clone, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub struct ExampleAccount {
    pub id: u64,
    pub other_stuff: Vec<u8>,
}

impl Fake for ExampleAccount {
    fn fake(seed: &Seed) -> ExampleAccount {
        let mut rng = IsaacRng::from_seed(&Self::to_rng_seed(seed));
        let size: u8 = rng.gen();
        ExampleAccount {
            id: rng.gen(),
            other_stuff: rng.gen_iter().take(size as usize).collect(),
        }
    }

    fn fake_version(_seed: &Seed) -> u64 {
        2
    }
}
```

and can be used in [`create_fake_account_packet()`]
(https://gitlab.com/Fraser999/Fake-Account-Packet/blob/master/src/main.rs#L55-72):

```rust
fn create_fake_account_packet(id: XorName, session_ids: &[SessionId]) -> StructuredData {
    let seed = get_seed(&id, &session_ids);
    let (public_key, private_key) = sign::keypair_from_seed(&seed);
    let type_tag = 0;
    let fake_account = ExampleAccount::fake(&seed);
    let data = serialisation::serialise(&fake_account).expect("Can't fail.");
    let version = ExampleAccount::fake_version(&seed);
    StructuredData::new(type_tag,
                        id,
                        version,
                        data,
                        vec![public_key],
                        vec![],
                        Some(&private_key))
        .expect("Can't fail.")
}
```

## Drawbacks

This presents a very basic implementation.  It requires the Vaults to generate throwaway
cryptographic keys in order to create a plausible fake - a relatively expensive operation.  For an
attacker just looking to do a DOS attack, this might provide a vector.  Profiling may show that this
proposed defence may need to become more complex, e.g. monitoring and reacting to unusually high
rates of `Get` requests for non-existent data, or caching spare throwaway crypto keys during quiet
periods for later use in generating fakes.

## Alternatives

1. Don't respond to `Get` requests for non-existent data.  While this would be simple to implement
in Vaults and would be the optimal way to avoid allowing an attacker to make Vaults "do work", it
causes problems for legitimate Clients which request a non-existent account.  Clients currently are
guaranteed to receive a response to every request.  If this changes, Clients would need to take
extra care to not ask for non-existent data, or else handle a non-response (possibly through a
timeout).

2. Respond with `GetFailure` for non-existent data, but monitor the rate at which such requests are
passing by.  This would need to be handled by the Vault through which the Client is connected.  Once
an unacceptably high rate of `GetFailure`s is reached, the Client is dropped.  This seems fairly
simple to implement, but an attacker would be able to work around this by using many Clients.

## Unresolved questions

1. Should the seed be generated using all members' names from the close group, or just a subset of
closest ones?
