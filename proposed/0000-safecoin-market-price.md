- Feature Name: SafeCoin_Market_Price
- Type: New feature
- Related components: maidsafe_vault, maidsafe_client
- Start Date: 19-07-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

This RFC is based on the concerns I’ve read in the MaidSafe forum for the last six months. These are some of them:

* https://forum.safenetwork.io/t/lowest-put-price-discovery-and-new-safecoin-issuance/3819
* https://forum.safenetwork.io/t/cost-of-storing-data-on-safe/4285
* https://forum.safenetwork.io/t/the-price-of-safecoin-the-economics-behind-it/844

This RFC is only another point of view (maybe not doable, feel free to reject this RFC) that breaks a lot the main design. Maybe this RFC can be decomposed in several RFC’s, smaller and feasible.


# Motivation

My first concern is how much cost to store a chunk of data (cost in terms of USD,EUR or BTC). No one is able to answer this question accurately. I’m thinking in a true Offer/Demand market, where you establish how much you want to pay to store data, and in the other side, the vaults establish how much they want to get paid to store that data.

Another important aspect is the idea of pay once and store data forever. I can’t see how can be sustainable in the future. I’d like to see a system where you Put data into the Net as a renting service, like the Amazon Web Services.


# Detailed design

* No data stored in perpetuity
* Every chunk has an owner.
* Price to store and/or retrieve information in a price market way. (Offer and demand)
* Allow huge and micro payments at fixed computational time. (A 1 million coin transfer lasts the same as 0.0001 coin transfer)
* Minting new coins at constant rate. No farming rates.

When you put your own data into the net you’re the responsible for pay every month to keep the data in the cloud. When you pay the Net, you’re paying directly to the vaults that store the data. Which means that in each operation there’s a person who pays and others who get paid. If an owner sets his data as public, he has to pay for each ‘get’ operation done to his chunk.

When you want to store data, you specify a max price you’re willing to pay for PUT and GET/POST operations. In the other side, each vault has specified a min price it’s willing to get paid. Then, the data managers look for a N valid vaults that match the prices. A month later the data managers are responsibles to look for another valid vaults or the same if the price continues matching. Meanwhile you can change the prices you’re willing to pay next month. That way you can adjust to the market price. If there isn’t any vault that matches your price or if you don’t have enough funds in your account, you lose your data.

When anyone wants to retrieve or update the data, another transaction is generated. The data owner pays the vaults storing the data. To reduce the amount of transactions, only 1 of 100 Get/Post operations are paid, and the amount paid is multiplied by 100. For example, if the Get price is set to 0.001 coins, there’s a chance of 1/100 to get 0.1 coins in each get/post operation.

Example:
- Owner sets prices in his Account:
  * Put -> Max 0.021 coins/chunk per month.
  * Get/Post -> Max 0.00004 coins/chunk per operation.
- Vault A sets prices in his Account:
  * Put -> Min 0.024 coins/chunk per month
  * Get/Post -> Min 0.00003 coins/chunk per operation.
- Vault B sets prices in his Account:
  * Put -> Min 0.019 coins/chunk per month
  * Get/Post -> Min 0.00002 coins/chunk per operation.

In this case, only Vault B is eligible to store Owner data.
A Put operation will be done at average price of 0.020 = ( (0.021 + 0.019) / 2)
A Get/Post operation will be done at 0.000025 coins.

This system implies a lot of micro payments with decimal point. It’s necessary a micro ledger for each account. It could be done with Merkle Trees distributed in different nodes along the net.

With a ledger for each account, you can transfer a huge amount of coins or a few with the same computational cost. They’re only two annotations, one in the owner’s ledger and another in the vault’s ledger.

New coins are generated every X minutes to expand the monetary system to the max 2^32 coins. These coins are distributed randomly to an active vaults who is connected more than H hours to the Net, or that have at least a R net rating.

# Drawbacks

* A user has to check every month the average price into the net to adapt his price to the price market to not lose data.
* No deduplication, each chunk has an owner.

# Alternatives

* The official alternative has the threat of an abrupt SafeCoin price increase in the secondary market (i.e. exchanges) could discourage the PUT operations due a FIAT price (in terms of EUR,USD or even BTC) too high.

# Unresolved questions

* How to find new vaults matching prices in ‘Churn events’
* Coin Distribution to app developers, and core developers.
* Find a way to generate a ‘winner’ every X minutes
