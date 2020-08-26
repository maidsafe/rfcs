# Locked DataChains

- Status: proposed 
- Type: enhancement
- Related components: data_chain
- Start Date: 01-09-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this) 
- Supersedes: None
- Superseded by: N/A

## Summary

A potential issue with data chains is the ability for old keys to be re-introduced. Old keys are analegous with "toxic waste" in snark type bootstrapping, i.e. they need to be destroyed. In our case these keys need to become irrelevent as destruction is not provable with digital information (copies can always exist and should be assumed to). Another issue is data_chain size, which does grow and may become cumbersome even though it splits across the network. This RFC addresses these points and more.

## Motivation

* Secure blocks in place and force immutability of the chain.
* Eradicate the ability to remove blocks.
* Make old keys irrelevent and useless to use in chain manipulation.
* Allow compression of the chain.
* Secure chain checkpoints, which allows sharing of the chain structure in approx Olog(n) parts where n is network population / average group size. This may be further reduced.

## Detailed design

A data chain can be seen as a huge lazy accumulator or as randomly delivered proofs of a blocks validity. This lazy approach is vital in decentralised networks at scale, and is very useful, however, it does make some validation difficult. The first step is to allow the blocks to re-organise in a chain and have data blocks maintained betwen two distinct churn events (full link block nodes). This is achieved by the following:

## React to a churn event
_This process is a naive approach and should be improved at implemementation with further tests._

1. On churn event each node creates a `NodeBlock`. 
2. The `NodeBlock` for a link is extended to be a hash of the current group, previous (valid) group and all `Blocks` in between. 
3. A node will select all `BlockIdentifier`s that are currently valid and move those that are not, past this point in the chain. 
4. These valid `Block`s are sent along with the Link `NodeBlock` to all group members.

## On reciept of a link `NodeBlock`

_This process is a naive approach and should be improved at implemementation with further tests._ 

1. Confirm partial chain is valid. 
2. If there are `Block`s we do not have then insert each new valid `Block` in our chain, removing any partial duplicate `Block` (implementation dependent, we can simply use the contained proofs to complete our own partial `Block`)
3. If the insertion created a new `LinkDescriptor` then send this to all group member nodes with the `Block`s we added.
4. Compress the chain by removing the `Proof`s for data blocks. 

## Drawbacks

This does cause extra traffic on a churn event as we add data to the normal `NodeBlock`. It is felt this will be made more efficient during implementation.

## Alternatives

There is a possibility of implementing a seperate structure in the form of a merkle tree to lock checkpoints in place, this could be extended to every churn event but would likely be wasteful.

## Unresolved questions

The process of sending the `Block`s with every churn event is wasteful and could be made much more efficient. It is assumed as this is implemented and furnished with a thorough test suite that those efficiency changes will be made.
