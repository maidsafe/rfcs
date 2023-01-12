# Safe Network Token Distribution

-   Status: Proposed 
-   Type: White Paper
-   Related to: Safe Network Tokens, DBCs, Governance, Nodes and Farming, MaidSafeCoin
-   Start Date: 23 June 2022
-   Discussion: https://safenetforum.org/t/rfc-0061-safe-network-token-distribution/
-   Supersedes: This proposal supersedes, in part, the original [Project Safe White Paper](https://github.com/maidsafe/Whitepapers/blob/master/Project-Safe.md) 

## Summary
This RFC sets out how the Network's utility tokens will be distributed at the inception of the Network, and how they are made available to contributors over time.

## Conventions
-   The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
The Safe Network has a utility token which allows for the exchange of storage, bandwidth and compute resource between node operators and users wishing to store data on the Network. These are Safe Network Tokens (SNT).

They also act as a means to fund and reward other contributions that provide utility and value to people who use the Network, and wider society, such as open source software development, sites, services, and publicly accessible data.

This paper sets out how these tokens will be distributed, and how they are made available to contributors.

## Detailed design

### Maximum Supply
There will be a total maximum of 4,525,524,120 whole SNT created over the Network’s lifetime.

#### Subunits
Each whole SNT can be subdivided 10<sup>9</sup> times, thus creating a total of 4,525,524,120,000,000,000 available subunits.

### Data Payments
Users wishing to store data on the Network, or edit existing data, pay the Network to do so in Safe Network Tokens. A Data Payment is made upfront and there are no ongoing costs to maintain data on the Network after this payment—content is made perpetually available after this one-time fee.

These Data Payment fees are immediately redistributed by the Network as follows:
- 85% is paid to qualifying node operators as a [Resource Supply Reward](#resource-rewards)
- 15% is remitted as [Network Royalties](#network-royalties)

### Resource Supply Rewards
Nodes provide data storage, bandwidth, and compute resources to the Network.

If a Node reliably and verifiably stores the data they are entrusted over an extended period, and serves it a timely manner when requested, they qualify to receive Resource Supply Rewards through virtue of meeting the required Node Age.

Resource Supply Rewards are automatically distributed by the Network to the operators of these nodes when a [Data Payment](#data-payments) is received by the Section in which they reside.

### Network Royalties 
Network Royalties are a mechanism through which software development, services, and data which provide value to people that use Network, benefit wider society, and meet the objectives of the project, can be can be meaningfully rewarded and sustainably funded.

Network Royalties are paid in support of the following areas.

#### Core Protocol Development
Individuals, teams, and businesses that support, research, design, develop, and maintain the open source software protocol of the Network and enable its ongoing operation, enhancement and security can become eligible for Network Royalties via the [Foundation](#subject-to-forthcoming-papers)'s [Developer Program](#subject-to-forthcoming-papers).

#### Client Application and Service Development
Operators and developers of software applications, platforms, sites, and services, that run on, and provide utility to, users of the Network can become eligible for Network Royalties via the Foundation's Developer Program. 

#### Public Data Accessibility
Creators, publishers, and curators of data that is made publicly and freely available for the common good, can become eligible for Network Royalties via the Foundation's [Data Commons Program](#subject-to-forthcoming-papers). 

#### Governance and Administration
In order to provide adequate, sustainable and transparent [governance](#subject-to-forthcoming-papers) to the Network and provide the required administration of the Developer and Data Commons programs, Network Royalties will also be used to cover the costs associated with the operation and work of the Foundation, and the distribution of Royalties.

### Distribution of Royalties
In accordance with the needs of the Network, and its ability to meet its objectives, the Foundation will oversee the distribution of [Royalties](#network-royalties) via the Developer and Data Commons Programs, through the following methods:

#### Grant Making 
Royalties may be paid in the form of grants to fund new areas of research, prospective  development of software, services, or other activities. 

#### Rewards
Participants in the Developer and Data Commons Programs may also be rewarded inline with the utility and value of their contributions, endeavours, services, to the users of the Network and the project's objectives in a given period. This may be in the form of one-off payments or regularised on-going funding such as a Service Level Agreements (SLA).

#### Ad Hoc Payments
The Foundation may also make ad hoc payments to fulfil its objectives and remit, its governance and regulatory obligations, and in order to cover the costs of administering and distributing Network Royalties.

Grants, Rewards, and ad hoc payments will be made from the [Network Royalties Pool](#network-royalties-pool), with any unspent or unclaimed funds in a given period returned to the pool for further distribution.

#### Automated Direct Distribution
It is an aspiration that the Network have the ability to automatically distribute Royalties, reducing both the time for recipients to be paid and the costs associated with administration. Autonomous distribution is subject to ongoing research for protocol development.

It is assumed that automatically distributed Network Royalties would be paid by the Network from Data payments directly at source, without entering the Network Royalty Pool.


### Genesis Supply
At the inception of the Network a Genesis Supply of 1,357,657,236 whole SNT will be issued. This represents 30% of the [Maximum Supply](#maxiumum-supply).

#### Distribution of Genesis Supply
The [Genesis Supply](#genesis-supply) of SNT will be distributed as follows:

##### MaidSafeCoin Holders
MaidSafeCoin is a proxy token issued as part of a crowd-sale in April of 2014 that supported the development of the Network and also allowed buyers to pre-purchase SNT ahead of launch, enabled via a 1:1 swap after the inception of the Network. This applies to both the original coins issued on the Omni layer ([MAID](https://www.omniexplorer.info/asset/3)) and the ERC-20 version ([eMAID](https://etherscan.io/address/0x329c6E459FFa7475718838145e5e85802Db2a303)).

Holders of MaidSafeCoin will collectively be allocated 452,552,412 SNT.

**This represents 10% of the Maximum Supply.**

Tokens will be distributed to MaidSafeCoin holders in the form of an airdrop, with each MaidSafeCoin entitling the bearer to one SNT.

##### MaidSafe Shareholders
Each company share of Maidsafe.net Limited will entitles the bearer to 111.5028029 SNT, resulting in shareholders being allocated 226,276,206 SNT. 

**This represents 5% of the Maximum Supply.**

Tokens will be paid out to shareholders in three instalments over the period of a year following the launch of the Network. Any unclaimed shareholder funds will be held by the Foundation indefinitely.


##### Network Royalties Pool
Out of the Genesis Supply, 678,828,618 SNT will be allocated to a Network Royalty Pool and distributed as [Network Royalties](#network-royalties).

**This represents 15% of the Maximum Supply.**

### Emission of Remaining Tokens
The remaining 3,167,866,884 of the Maximum Supply will be emitted by the Network as Resource Supply Rewards(#resource-supply-rewards) over an extended period, at a rate corresponding to Network growth as measured by the volume of data stored by its nodes.

**This represents the remaining 70% of the Maximum Supply.**

It is assumed that this process, which is subject to further research and development, will not be in place a the inception of the Network, but will be implemented via a future Network update. 

Emission will gradually increase the circulating supply of SNT over an extended period until the Maximum Supply is reached. It is anticipated that this will take many years, or even the lifetime of the Network.


## Drawbacks
There are drawbacks to a foundation overseeing and handling any size of fund, namely:

- Security implications of holding tokens
- Costs associated with administration
- The centralising effects of doing so

While these deserve to be highlighted and discussed, they can also be mitigated through due consideration to appropriate governance and through the development of automated distribution processes as noted in this paper.


## Alternatives

### Remaining Token Distribution Proposal
An alternative considered and debated in an earlier revision of this RFC included a fallback position of the Foundation distributing the remaining 70% of the token supply. 

This would mean it potentially handling 85% of the total supply. 

While it was conceivable that this could be done equitably and in line with its aims, it also was rightly deemed to present an undue security risk, administrative burden, and centralising pressure that would take some years to unwind.

Given that we now feel confident that automated Network distribution of these tokens is possible in a straightforward and secure manner, and that this could be initiated via a Network update at such time as it has been adequately tested and proven, it seems appropriate to shelve the original fallback. 

### Resource Supply Rewards
Note that the term [Resource Supply Rewards](#resource-supply-rewards) is an alternative name for what was previously called Farming Rewards. This reflects advice to use more precise terminology to describe the economic mechanism.
 
## Unresolved Questions

### With Regard to Subunits
As we are currently finalising the design of the Digital Bearer Certificates (DBC) system, the exact number of sub-units may be increased to provide further divisibility. This is subject to the results performance testing and security analysis.

### With Regard to MaidSafe Shareholder Payouts
- We are yet to define precisely what event constitutes the "launch" of the Network, thus triggering the process of Shareholder Payouts.
- We are yet to determine if there is a requirement for the Foundation to hold any unclaimed shareholder funds indefinitely, or if it can be time limited.
  
### Subject to Forthcoming Papers
The Foundation is a Swiss non-profit organisation incorporated to support the security, privacy, and sovereignty of personal data and communications, the resilience and global accessibility of public data, the pursuit of a free and open Internet for the public good, and the ability of individuals and businesses to trade goods and services online without the need for middlemen, through the promotion and stewardship of the Safe Network protocol, it’s ecosystem, and related distributed ledger and computing technology

The Foundation's governance structure, Developer Program, and Data Commons Program will be addressed in forthcoming papers via the RFC process, along with those detailing the specifics of the technical design of Safe Network Token and DBC system.
