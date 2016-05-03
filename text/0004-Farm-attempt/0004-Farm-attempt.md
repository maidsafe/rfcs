- Feature Name: Farm attempt
- Status: agreed
- Type: new feature
- Related components: maidsafe_vault
- Start Date: 24-06-2015
- Issue number: #25

# Summary

This proposal outlines an initial design to test the efficiency and security of farming attempts. This involves making use of the Farming Rate (FR) which is the rate at which farming attempts are made. It is assumed the farming rate will include payments to care development, content producers and most significantly farmers (or providers of the network resource).

# Motivation

To ensure network stability and resource availability a reward mechanism is required to ensure the use of resource is balanced with the supply of resource.

# Scope of work

This implementation will be solely in the vault library. This will include several steps. The client lib will require to be aware of the actual safecoin type (again `StructuredData`).

# Detailed design

On receipt of a `Get` request for `ImmutableData` the `DataManager` calculates the SHA512 Hash of the `name` of the `ImmutableData` concatenated with the`PmidNode`. This result is then modulo divided by the farming rate (FR). The rate attempts should divide the award between 4 distinct groups (for clarity, this algorithm is applied to all `PmidNodes` that hold this data element in the current group).

## Rate attempts

1. PmidNodes -> tested at 100% of FR [i.e. Farming rate * 1]
2. App Developer -> tested at 10% of FR [i.e. Farming rate * 1.1]
3. Publisher -> tested at 10% of FR [i.e. Farming rate * 1.1]
4. Core development -> tested at 5% of FR [i.e. Farming rate * 1.05]

## Payment address

1. PmidNodes -> Registers an Optional (if set by user) wallet address on any store of data to it
2. App Developer -> App developers will include wallet address in any `Get` request
3. Publisher -> As data is stored a wallet address of the owner is stored if this is first time seen on the network (stored in DM account for the data element)
4. Core development -> Initially every node will be aware of a hard coded wallet address for core development. This will likely lead to a multi-sign wallet.

## Farm process

When the initial farming test above is true (the modulo division `==0` ) then the `DataManager`s `Post` a Farm request message to the group closest to the combined Hash (i.e. `Hash(ImmutableData + PmidHolder)`). This request includes the original hashes + hash of chunk requested + current farming rate of the group as well as wallet address. This is confirmed at the receiving group and they check for the existence of a safecoin space (i.e. there is no safecoin with this name already). If this is successful then a further check is made to confirm there is a `DataManager` group (implicit as routing does this). A safecoin is then created and a message sent to the wallet holder of the request. All these requests must be digitally signed and confirmed at receiving end (implicit in sentnel).

# Drawbacks

Left blank for discussion

# Alternatives

Initially there was no safecoin and the network would have been built in a quid pro quo manner. Third involved users requiring a vault to store data and the user then being allocated that amount of data to store. It was rather inflexible and involved a tremendous amount of logic. It was an alternative. It should be noted the very original designs did include a digital currency which would have suited this purpose perfectly as safecoin now will.

# Unresolved questions

1. Link farming rate to resource cost (what users must pay). This will form a separate RFC.
