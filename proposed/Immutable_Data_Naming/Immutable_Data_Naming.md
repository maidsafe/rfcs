- Feature Name: ImmutableData naming base on type
- Type: New Feature
- Related Components: [safe_vault](https://github.com/maidsafe/safe_vault), [safe_client](https://github.com/maidsafe/safe_client), [routing](https://github.com/maidsafe/routing)
- Start Date: 02-11-2015
- RFC PR: 
- Issue number: Proposed - #

# Summary

This RFC outlines the system components and design for how the three immutable data types (Normal, Backup and Sacrificial) get calculated and handled on the SAFE Network.

# Motivation

## Rationale

The SAFE Network keeps multiple copies of one original data not only for the purpose of avalability but also for secure reason.  It has been defined three different types currently : Normal, Backup and Sacrificial.  They are currently having the same name (hash of the content), which means it will be the same group of DataManager to handle them.

Making three different types copy bearing different but determinic name will increase the difficultiy of any possible attack, as it will be three different groups, instead of just one group, need to be tackled at the same time.

## Supported Use-Cases

For client perspective, the use-case keeps the same : storing or fetching immutable data.  Though client may need to be aware of the existing of different types and calculations of name. 

## Expected Outcome

The different immutable data type copy bearing different name, which will be handled by different DataManager group.

# Detailed design

## Overview

The name of a immutalbe data copy will be based on it's type and in different hash order :

Normal : normal_name = hash(im.content), handled by DM(normal_name), 4 copies on the pmid_nodes picked up by that group (one copy on each node)

Backup : backup_name = hash(normal_name), handled by DM(backup_name), 4 copies on the pmid_nodes picked up by that group (one copy on each node)

Sacrificial : sacrificial_name = hash(backup_name), handled by DM(sacrificial_name), 4 copies on the pmid_nodes picked up by that group (one copy on each node)

The MaidManager of the client issuing the put request will charge only 4 copies of the data, as the backup and sacrificial copies are allowed to be removed from the SAFE network according to the network status.

## Implementation Details


## Planned Work

1. Vault
    1. DataManager
    1. MaidManager
    1. put flow refactoring
    1. get flow refactoring

1. Routing
    1. ImmutableData sanity check

1. Client
    1. Put
    1. Get


# Drawbacks

None identified, other than increased complexity of Vault and Client codebase.

# Alternatives

1. The SAFE network itself, is able to be free of carrying out any naming calculation and handling based on types, as long as client be aware of such and fire requests bearing the different type-dependent name.  This will have the least impact to the current code base, however the client app must need to be aware of that and carry out its duty. It also leaves an option (probably good) when the client app decides only one of the type will be enough.


# Unresolved Questions

1. It is notionable that to reduce the code coplexity, the portal DataManager group of an immutable data shall still be DM(normal_name). Then such group can forwarding the requests to pmid_nodes closest to normal_name, backup_name, sacrificial_name.
However, due to the facts the pmid_nodes all already closest nodes to the data_manager, such forwarding mechanism may not be able to securse a highly diversity of the distribution. i.e. some pmid_nodes may holding different type copies at the same time.

2. Fraser argued that the hash of the hash paradigm shall not be used, since this could yield the same group for the normal, backup and sacrificial copies (the probability of this increasing to 1 as the network size decreases).  He proposed to XOR the chunk ID ( hash(content) ) to yield each group. While this should be more efficient than repeated hashing, more importantly it should yield groups which don't intersect (except for tiny networks).
Essentially the "normal" group will be XOR with 000...000 i.e. equivalent to the ID.
The sacrificial group will be XOR with fff...fff, i.e. the furthest location in the address space. And the backup will be in the middle; i.e. XOR with 800...000.


# Future Work



# Appendix

