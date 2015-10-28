- Feature Name: Safecoin implmementation
- Type new feature
- Related components safe_vault, safe_client 
- Start Date: 12-10-2015 
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Full implementation of safecoin v1.0. This RFC brings together the following RFC's
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
which at this time is simply a name given to this overall unit of measure (as far as this RFC is conscerned). 


## Data/chunk size consideration

Data of varying sizes is uploaded to the SAFE network. Immutable data can be up to 1Mb and StructuredData
can be up to 100Kb. To simplify the algorithm and also avoid bad players attempting to swamp the 
network with tiny data elements as a reduced cost, we consider all data uploaded as a single unit. 
These data units, we know are at most 1Mb and we should encourage developers to maximise their 
use of the network by storing as close to this value as possible. 

In this design each upload (PUT) will incur the same cost, i.e. 1 unit. 

To facilitate this design we will create an internal name to measure this unit of store. This will
be referred to as a StoreCost in this document.


##Establishing Farming Rate 

This section will introduce the following variables:

Total primary chunks count     == TP
Total sacrificial chunks count == TS

These values are amortised across the network and across groups close to each other. The farming 
rate is therefore defined as a simple algorithm:

`FR = (TP / (TP -TS))`

This will allow the network to increase farming when sacrifical data is decreasing and will satisfy 
a balancing algorithm which can be measured during network tests. 

It should be noted that as the network grows farming rate decreases, as it should. This will push 
the design of the archive nodes to ensure the number of chunks active in the network is not excessive.
Archive nodes will be a further RFC and should allow farming rates to have a natural minimum.
 
##Establishing StoreCost

This is an upgrade to RFC [0005](https://github.com/dirvine/rfcs/blob/safecoin_implementation/agreed/0005-balance_network_resources.md) the initial StoreCost

and consequent farming reward is 1 safecoin for the first Get and exponentially decreases from that point. 

The inital Put cost has to be related of the number of clients verses number of vaults (resource providers)
in SAFE this will be achieved by the following :

Vaults have a Farming Rate (FR)
Vaults can query the total number of client (NC) accounts (active, i.e. have stored data, possibly paid)
Vaults are aware of GROUP_SIZE

The calculation therefore becomes a simple one (for version 1.0)

`StoreCost = FR / (GROUP_SIZE / NC)`

Therefore a safecoin will purchase an amount of storage equivalent to the amount of data stored (and
active) and the current number of vaults and users on the network. 

##Farm request calculation

The farming request calculation is also a simple affair, but must mitigate against specific attacks
on the network. These include, but are not limited to:

1. Continual Get against known data on a vault
2. Attempted targeting of farm rewards

The farming attempt will include Data Manager addresses (to differentiate a farm request from the 
data name itself). It will also include the chunk name and the name of the ManagedNode (PMID node)

This process is outlined as:

1. Get request for Chunk X is received.
2. The DataManagers will request the chunk from the ManagedNodes holding this chunk.
3. The ManagedNodes will send the chunk with their wallet address included. 
4. Teh DataMangers will then select take the address of each DataManager in the QUORUM.
5. This is then hashed with the chunk name and PmidHolder name.
6. If this `result` % Farming rate (modulu divides) with a zero value as the answer then 
   This data is sent to the group who are closest to `result`
   This request is a POST message as a safecoin request
   IF there is a safecoin available of the name `result` then
   The safecoin is created and the onwer set to the wallet address provided in the `result` packet
   The safecoin close group then send a receipt message to the wallet address to inform the user 
   of a new minted safecoin allocated to them. 

##Bootstrap with clients

Although there has been hostility form the community with regard to `something for nothing` approach, there is a necessity for a bootstrap mechnism. As no safecoin can be farmed untill data is uploaded there is a cyclic dependency that reuqires resolution. To overcome this limitation this RFC will propose that every new account creatred also is charged up with 50 safecoin per account. This may be temporary and only used in test-safecoin, but is is likely essential to allow this for the time being. It may be a mechanism to kickstart the network as well.

# Drawbacks

These will be added during the review process and will include any concerns form the community forum 


# Alternatives

Initially there was no safecoin and the network would have been built in a quid pro quo manner. 
This involved users requiring a vault to store data and the user then being allocated that
amount of data to store. It was rather inflexible and involved a tremendous amount of logic. 
It was an alternative. It should be noted the original designs did include a digital
currency which would have suited this purpose perfectly as safecoin now will.

There is an alternative approach outlined [here](https://forum.safenetwork.io/t/safecoin-divisibility/4806/68) which introduces an alternatice coin for artists and app developers. This RFC does not limit this proposal and leaves the way open for such an implementation. 

# Unresolved questions

The Application developer rewards are seen as a good start to pay creators of applications on the 
app popularity, measured via it's use. This design incorrectly identifies the measure of use as the
number of GET requests the app carries out. A better solution should be found for this measure.

Some have identified an app may 

#Implementation overview

##Farming rate method

```
fn farming_rate() -> i64 {
  let result = total_primary_chunks / (if total_primary_chunks >= total_sacrifical_chunks { total_primary_chunks - total_sacrifical_chunks } else { 1 }
  if result < (if total_primary_chunks >= total_sacrifical_chunks { total_primary_chunks - total_sacrifical_chunks } else { 1 } { i64::MAX } else { result }

}
```
##Client Put (StoreCost)

ClientManager

```
if Put && key.is_in_range() { // we are client manager
    if ! key.in_account_list {
        Error::NoAccount;
    }
    let cost = vault.farming_rate / (GROUP_SIZE / account.list.len())
    if cost > account_balance {
        Error::NotEnoughBalance;
    } else {
    account_balance = account_balance - cost;
    }
    routing.Put(put_data, data.name()); // send actual network put to datamangers responsible for the chunk name
}
```
##Client account creation, addition

```
fi new_account(name) {
  for (0..50) {
    account_balance.name += 1 / (farming_rate() /(GROUP_SIZE/ if account.len() ==0 { 1 } else { account.len() }
  }
}
```
