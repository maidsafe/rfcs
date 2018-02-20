# Safecoin Implementation

- Status: proposed
- Type: new feature
- Related Components: safe_vault, safe_client
- Start Date: 12-10-2015
- RFC PR: #60
- Issue Number: Proposed - #61
- Discussion: https://github.com/maidsafe/rfcs/issues/61
- Supersedes:
- Superseded by:

## Summary

Full implementation of safecoin v1.0. This RFC brings together the following RFCs
[Farm Attempt](https://github.com/maidsafe/rfcs/blob/master/text/0004-farm-attempt/0004-farm-attempt.md)
[Balance Resources](https://github.com/maidsafe/rfcs/blob/master/text/0005-balance-network-resources/0005-balance-network-resources.md)
In addition this RFC will attempt to calculate the existing magic numbers used in previous implementations.

## Motivation

Safecoin is a valuable part of the SAFE network and allows resource providers to be paid by users of
that resource. Users in this case are content producers, or those who upload data to the network.

This allows the network to function at, hopefully, the lowest possible cost of resource as the
resources provided by farmers (providers of resource) are designed to be unused resources. The reward
will likely be advantageous to people and provide encouragement. In the early days these rewards may
be significant as the network is in a state of finding an equilibrium between provision and
consumption of resources.

This RFC will cover farming and payments as well as define the wallet interface for application
developers.

## Detailed design

Initially the cost of resources in some unit must be identified. There are options to measure these
units in disk space, CPU, bandwidth and more. It is much simpler to define these units as a safecoin,
which at this time is simply a name given to this overall unit of measure (as far as this RFC is concerned).

### Data/chunk size consideration

Data of varying sizes is uploaded to the SAFE network. Immutable data can be up to 1Mb and StructuredData
can be up to 100Kb. To simplify the algorithm and also avoid bad players attempting to swamp the
network with tiny data elements as a reduced cost, we consider all data uploaded as a single unit.
These data units, we know are at most 1Mb and we should encourage developers to maximise their
use of the network by storing as close to this value as possible.

In this design each upload (PUT) will incur the same cost, i.e. 1 unit.

To facilitate this design we will create an internal name to measure this unit of store. This will
be referred to as a StoreCost in this document.

### Establishing farming rate

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
the rate to increase as we lose sacrificial chunks.  We also want to ensure that farming stops if
the sacrificial count is greater than or equal to the primary count.

This means the farming divisor is defined as:

```rust
if TP > TS {
    FD = TP / (TP - TS)
} else {
    FD = maximum possible value
}
```

yielding

```rust
if TP > TS {
    FR = 1 - (TS / TP)
} else {
    FR = approximately 0
}
```

As with many aspects of the network, in the early days we expect the network and the distribution of
chunks to be fairly unbalanced, but over time this farming rate should tend towards a consistent
figure across the whole network.

Since the farming rate decreases as the network grows, it will push the design of the archive nodes
to ensure the number of chunks active in the network is not excessive.  Archive nodes will be a
further RFC and should allow farming rates to have a natural minimum.

### Establishing StoreCost

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

### Farm request calculation

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

### Safecoin Management

Each safecoin is represented as a piece of data held by the group closest to it's ID. Safecoin data structure is defined as:
'''rust
ID: 64 bytes
OWNER: 64 bytes
'''
The ID of a safecoin is 64Bytes long, with the most meaningful 32 bits being sequenced index starts from 0 to 4294967295, and the left over part to be fullfiled with all zeros or other pre-defined pattern (to allow coin division).
The OWNER of a safecoin is the wallet address provided by the pmid_node as mentioned above.

The Safecoin Management group can only approve a farming request when no such targeted safecoin data has been created before.

When being asked to transfer the ownership, the request transcript must provide a valid signature that can be verified by the stored OWNER, which is actually a public-key. And the owner will be updated to the new owner.

When being asked to burn a coin, the request must be signed by the current owner and forwarded through that owner's Client Manager group (which will increase allowed storage space at the same time). The piece of safecoin data will then be removed.

### Account Management

An Account Management group is a group of nodes closest to a user's wallet address. It is responsible for that user's safecoin relation activities: rewarding, transferring or discarding.
A user's safecoin account is defined as :
'''rust
OWNER: 64 bytes
COINS: Vec<SAFECOIN_ID>
'''

1. rewarding : when received notification a safecoin has been successfully farmed, record the ID of that coin into the account
2. transfer out : remove certain number of coins from the account record, and notify the receiver's account group and the chosen safecoins' management groups of the ownership transferring.
3. transfer in : when being notified by the sender's account group and the safecoin management group, the correspondent safecoin's ID will be inserted into the record.
4. discarding : This is a special case that no receiver has been specified. The safecoin will be removed from the account and the chosen safecoins' management groups will be notified with a burning request.

### Bootstrap with clients

Although there has been hostility from the community with regard to "something for nothing" approach, there is a necessity for a bootstrap mechanism. As no safecoin can be farmed until data is uploaded there is a cyclic dependency that requires a resolution. To overcome this limitation this RFC will propose that a user account is composed of two parts: safecoin_account and storage_account, every new account created is initialised with safecoin_account holding 0 coins but storage_account holding 50 safecoin equivalent storage allowance. This may be temporary and only used in test-safecoin, but it is likely essential to allow this for the time being. It may be a mechanism to kickstart the network as well.


## Drawbacks

These will be added during the review process and will include any concerns form the community forum.

## Alternatives

Initially there was no safecoin and the network would have been built in a quid pro quo manner.
This involved users requiring a vault to store data and the user then being allocated that
amount of data to store. It was rather inflexible and involved a tremendous amount of logic.
It was an alternative. It should be noted the original designs did include a digital
currency which would have suited this purpose perfectly as safecoin now will.

There is an alternative approach outlined [here](https://forum.safenetwork.io/t/safecoin-divisibility/4806/68) which introduces an alternative coin for artists and app developers. This RFC does not limit this proposal and leaves the way open for such an implementation.

## Unresolved questions

The application developer rewards are seen as a good start to pay creators of applications on the
app popularity, measured via its use. This design incorrectly identifies the measure of use as the
number of GET requests the app carries out. A better solution should be found for this measure.

Some have identified an app may

## Implementation overview

### Farming rate method

```rust
fn farming_divisor() -> u64 {
    if total_primary_chunks > total_sacrificial_chunks {
        total_primary_chunks / (total_primary_chunks - total_sacrificial_chunks)
    } else {
        u64::MAX
    }
}
```

### Client Put (StoreCost)

```rust
fn store_cost() -> u64 {
    // The number of active client accounts must be at least 1 (or storing will be free!).  This
    // should always be the case, since at least the requesting client's account is active.
    assert!(number_of_active_client_accounts > 0)
    number_of_active_client_accounts / (farming_divisor() * GROUP_SIZE)
}
```

ClientManager

```rust
if key.is_in_range() { // we are client manager
    if !key.in_account_list {
        return Error::NoAccount;
    }
    match operation {
        Put => {
            if store_cost() > storage_balance {
                Error::NotEnoughBalance;
            } else {
                storage_balance -= store_cost();
            }
            // send actual network put to DataManagers responsible for the chunk name
            routing.Put(put_data, data.name());
        }
        Reward, TransferIn => {
            safecoin_account.add(new_coin);
        }
        Convert, TransferOut => {
            let removed_coins = safecoin_account.remove(quantity);
            if receiver.is_some() {
                // Notify the receiver
                routing.Post(removed_coins, receiver);
            } else {
                // This is a convert operation
                for (0..quantity) {
                    storage_balance += 1 / store_cost();
                }
            }
            for coin in removed_coins {
                // Notify each coin's management group
                routing.Post(compose_transfer_msg(coin, receiver), coin.id);
            }
        }
    }
}
```

### Client account creation, addition

```rust
fn new_account(name) {
    let mut tuple = (name, storage_balance, safecoin_account);
    for (0..50) {
        tuple.storage_balance += 1 / store_cost();
    }
    account_list.push(record);
}
```

### Safecoin Manager
```rust
if coin.is_in_range() { // we are the manager of that coin
    match operation {
        Farm => {
            if coin_list.has(coin) {
                return Error::NotFree;
            }
            coin_list.push(generate_coin(coin, requester));
            // send notification to the requester
            routing.Post(coin, requester);
        }
        Transfer => {
            if !coin_list.has(coin) {
                return Error::NoSuchCoin;
            }
            if coin_list.get(coin).owner != requester {
                return Error::InvalidRequest;
            }
            coin_list.update(coin, requester, receiver);
            // send notification to both the requester and receiver
            routing.PostResponse(coin, requester);
            routing.Post(coin, receiver);
        }
        Burn => {
            if !coin_list.has(coin) {
                return Error::NoSuchCoin;
            }
            if coin_list.get(coin).owner != requester {
                return Error::InvalidRequest;
            }
            coin_list.remove(coin);
            // send notification to the requester
            routing.PostResponse(coin, requester);
        }
    }
}
```
