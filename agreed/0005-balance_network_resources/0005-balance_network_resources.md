- Feature Name: Balance network resources
- Type new feature
- Related components maidsafe_vault
- Start Date: 25-06-2015
- Issue number: #26

# Summary

This proposals intent is to provide a balance for Farming Rate verses purchase price of network
resources that users are charged. The farming rate is calculated by the network in another RFC.
As the farming rate increases, farmers earn less as more `Get` requests per attempt is required.
As the farming rate increases then it follows users pay less (as the network is oversupplied) for
resources. The key issue is to identify the link between these two numbers. It is assumed
all chunks are 1Mb in this proposition, this is very realistic in terms of calculation and
any error favours the user.

# Motivation

This decentralised autonomous network must encourage resource providers. This is anticipated
to be similar to skype, bittorrent etc. in that people will provide resources, simply because
they are getting something from the network. This proposal enhances safecoin to provide an
additional encouragement. That incentive is the payment of providers of resources to include
application development, content availability and importantly resources, in terms of disk space,
cpu processing, memory, bandwidth etc. in fact everything required to provide data availability
(and compute) to all users, irrespective of the use or users.

Many confuse any out of network price fluctuation would in fact alter the network balancing process,
in fact it is unrelated as the balance simply increases reward when space is needed and reduces when
space is abundant. There is no relation to the cost of safecoin and network balancing, as long as
a safecoin can change hands for a cost then the number of safecoin per unit of fiat is handled by
the network.

# Detailed design

A very simple approach is to use a fixed approach and measure via testing. It is assumed there will
be considerably more reads from the network than stores of data. The cost per MB, electricity, B/W
and size of storage are all factors in this algorithm. Therefore testing and sampling of a running
network is essential. To begin the simple approach is best.

`int cost_per_Mb = 1/(fr^(1/5));`  (5th root, integer value)

Where 1 is a 1MB chunk. It is assumed the figure will alter under scrutiny of network testing, but
this is assumption (based on usage increase (X10 pa total), price decrease (1/2 pa) etc. over time)
as well as b/w usage, which widely varies across countries at the moment.

The FR is the local farming rate known to the `ClientManager` and may slightly vary through time across
the network. This is expected and as the network grows the spread of cost will equalise as the binary
tree balances.

`ClientManagers` will expect at least a whole safecoin in advance, which is `burned` (i.e. the client
sends a `Delete` for a safecoin (which the `ClientManager` checks)). The balance of this payment is
reduced per `Put`. This `Delete` call will pass through `ClientManagers` who can immediately add the
balance and if an `Error` for this `Delete` is returned then reduce the balance and remove the account.

It is assumed clients will pay at least one safecoin to create an account. This payment will be converted immediately to Mb of storage space and the client can query this figure at any time.

# Drawbacks

1. This is a very simple approach to a balancing algorithm, it could be argued this could be modeled.

2. Initial feedback and analysis may not reflect network usage as new applications and use profiles
alter over time.

3. It is very likely the agreed algorithm will evolve over time, which may not be clearly understood by all
the community, therefore clear RFC proposals and discussions should certainly take place when such
change may happen.

# Alternatives

It is expected there may be, in fact, a substantially more accurate initial estimate or an algorithm
that is likely to be more capable of aligning with changing Farming Rates.

# Unresolved questions

Left open to review process to identify
