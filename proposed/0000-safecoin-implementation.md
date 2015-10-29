- Feature Name: Safecoin implementation
- Type: new feature
- Related Components: safe_vault, safe_client
- Start Date: 12-10-2015
- RFC PR: (leave this empty)
- Issue Number: (leave this empty)

# Summary

Full implementation of safecoin v1.0. This RFC brings together the following RFCs
[Farm Attempt](https://github.com/maidsafe/rfcs/blob/master/agreed/0004-Farm-attempt/0004-Farm-attempt.md)
[Balance Resources](https://github.com/maidsafe/rfcs/blob/master/agreed/0005-balance_network_resources/0005-balance_network_resources.md)
In addition this RFC will attempt to calculate the existing magic numbers used in previous implementations.

# Motivation

Safecoin is a valuable part of the SAFE network and allows resource providers to be paid by users of
that resource. Users in this case are content producers, or those who upload data to the network.

This allows the network to function at, hopefully, the lowest possible cost of resource as the
resources provided by farmers (providers of resource) are designed to be unused resources. The reward
will likely be advantageous to people and provide encouragement. In the early days these rewards may
be significant as the network is in a state of finding an equilibrium between provision and
consumption of resources.

This RFC will cover farming and payments as well as define the wallet interface for application
developers.

# Detailed design

Initially the cost of resources in some unit must be identified. There are options to measure these
units in disk space, CPU, bandwidth and more. It is much simpler to define these units as a safecoin,
which at this time is simply a name given to this overall unit of measure (as far as this RFC is concerned).

## Data/chunk size consideration

Data of varying sizes is uploaded to the SAFE network. Immutable data can be up to 1Mb and StructuredData
can be up to 100Kb. To simplify the algorithm and also avoid bad players attempting to swamp the
network with tiny data elements as a reduced cost, we consider all data uploaded as a single unit.
These data units, we know are at most 1Mb and we should encourage developers to maximise their
use of the network by storing as close to this value as possible.

In this design each upload (PUT) will incur the same cost, i.e. 1 unit.

To facilitate this design we will create an internal name to measure this unit of store. This will
be referred to as a StoreCost in this document.

## Establishing farming rate

This section will introduce the following variables:

- Farming rate                   == FR (0 < FR <= 1)
- Farming divisor                == FD (FD >= 1)
- Total primary chunks count     == TP (TP >= 0)
- Total sacrificial chunks count == TS (TS >= 0)

These values are amortised across the network and across groups close to each other.

Each successful GET will generate a unique identifier (see below for details).  This identifier will
be used as the dividend in a modulo operation with the FD as the corresponding divisor.  If this
operation results in `0` then farming attempt is successful.

Hence the farming rate is defined as:

`FR = 1 / FD`

In other words, if FD is 1, every attempt is successful and FR is 1.  If FD is 10, on average every
tenth attempt is successful and FR is 0.1.

Broadly speaking, we want the farming rate to drop as the number of chunks increases, but we want
the rate to increase if we start to lose sacrificial chunks.

For the first requirement, we can achieve this by having FD as the maximum of TP and TS (we'll call
this maximum total "MT").

For the second requirement, we want to reduce the FD if we have less sacrificial chunks than primary
ones.  This means the farming divisor is defined as:

```rust
if TS < TP {
    FD = MT - (TP - TS) + 1
} else {
    FD = MT + 1
}
```

Since the farming rate decreases as the network grows, it will push the design of the archive nodes
to ensure the number of chunks active in the network is not excessive.  Archive nodes will be a
further RFC and should allow farming rates to have a natural minimum.

This is a simplistic formula which will very likely need to be modified as more information about
the make up of the network becomes available.  For example, more weight might need to be given in
the case of loss of sacrificial chunks, so something like `FD = MT - 2 * (TP - TS) + 1` could be
used (adjusted so that FD remains >= 1).

## Establishing StoreCost

This is an upgrade to RFC [0005](https://github.com/dirvine/rfcs/blob/safecoin_implementation/agreed/0005-balance_network_resources.md) the initial StoreCost

and consequent farming reward is 1 safecoin for the first Get and exponentially decreases from that point.

The initial Put cost has to be related to the number of clients versus the number of vaults (resource providers).
In SAFE this will be achieved by the following:

Vaults have a farming rate (FR)
Vaults can query the total number of client (NC) accounts (active, i.e. have stored data, possibly paid)
Vaults are aware of `GROUP_SIZE`

The calculation therefore becomes a simple one (for version 1.0)

`StoreCost = FR * NC / GROUP_SIZE`

Therefore a safecoin will purchase an amount of storage equivalent to the amount of data stored (and
active) and the current number of vaults and users on the network.

## Farm request calculation

The farming request calculation is also a simple affair, but must mitigate against specific attacks
on the network. These include, but are not limited to:

1. Continual Get against known data on a vault
2. Attempted targeting of farm rewards

The farming attempt will include DataManager addresses (to differentiate a farm request from the
data name itself). It will also include the chunk name and the name of the ManagedNode (PMID node)

This process is outlined as:

1. Get request for Chunk X is received.
2. The DataManagers will request the chunk from the ManagedNodes holding this chunk.
3. The ManagedNodes will send the chunk with their wallet address included.
4. The DataManagers will then take the address of each DataManager in the QUORUM.
5. This is hashed with the chunk name and PmidHolder name.
6. If this `result` % farming divisor (modulo divides) yields zero then
   - This data is sent to the group who are closest to `result`
   - This request is a POST message as a safecoin request
   - If there is a safecoin available of the name `result` then
     - The safecoin is created and the owner set to the wallet address provided in the `result` packet
     - The safecoin close group then send a receipt message to the wallet address to inform the user
       of a new minted safecoin allocated to them.

## Bootstrap with clients

Although there has been hostility from the community with regard to "something for nothing" approach, there is a necessity for a bootstrap mechanism. As no safecoin can be farmed until data is uploaded there is a cyclic dependency that requires a resolution. To overcome this limitation this RFC will propose that every new account created is initialised with 50 safecoins. This may be temporary and only used in test-safecoin, but it is likely essential to allow this for the time being. It may be a mechanism to kickstart the network as well.

# Drawbacks

These will be added during the review process and will include any concerns form the community forum.

# Alternatives

Initially there was no safecoin and the network would have been built in a quid pro quo manner.
This involved users requiring a vault to store data and the user then being allocated that
amount of data to store. It was rather inflexible and involved a tremendous amount of logic.
It was an alternative. It should be noted the original designs did include a digital
currency which would have suited this purpose perfectly as safecoin now will.

There is an alternative approach outlined [here](https://forum.safenetwork.io/t/safecoin-divisibility/4806/68) which introduces an alternative coin for artists and app developers. This RFC does not limit this proposal and leaves the way open for such an implementation.

# Unresolved questions

The application developer rewards are seen as a good start to pay creators of applications on the
app popularity, measured via its use. This design incorrectly identifies the measure of use as the
number of GET requests the app carries out. A better solution should be found for this measure.

Some have identified an app may

# Implementation overview

## Farming rate method

```rust
fn farming_divisor() -> u64 {
    let bias_for_lost_sacrificial = if total_sacrificial_chunks < total_primary_chunks {
        total_primary_chunks - total_sacrificial_chunks
    } else {
        0
    };

    ::std::cmp::max(total_primary_chunks, total_sacrificial_chunks) - bias_for_lost_sacrificial + 1
}
```

## Client Put (StoreCost)

```rust
fn store_cost() -> u64 {
farming_rate() / (GROUP_SIZE / if account.len() ==0 { 1 } else { account.len())
}
```


ClientManager

```rust
if Put && key.is_in_range() { // we are client manager
    if !key.in_account_list {
        Error::NoAccount;
    }
    if store_cost() > account_balance {
        Error::NotEnoughBalance;
    } else {
        account_balance -= store_cost();
    }
    // send actual network put to DataManagers responsible for the chunk name
    routing.Put(put_data, data.name());
}
```

## Client account creation, addition

```rust
fn new_account_inital_safecoin(name) {
    for (0..50) {
        account_balance.name += 1 / store_cost()
    }
}
```
