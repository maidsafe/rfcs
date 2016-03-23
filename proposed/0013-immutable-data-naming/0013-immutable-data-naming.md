- Feature Name: ImmutableData naming based on type
- Type: New Feature
- Related Components: [safe_vault](https://github.com/maidsafe/safe_vault), [safe_core](https://github.com/maidsafe/safe_core), [routing](https://github.com/maidsafe/routing)
- Start Date: 02-11-2015
- RFC PR: #59
- Issue number: Proposed - #65

# Summary

This RFC outlines the system components and design for how the three immutable data types (Normal, Backup and Sacrificial) get calculated and handled on the SAFE Network.

# Motivation

## Rationale

The SAFE Network keeps multiple copies of a single ImmutableData chunk, not only for availability but also for security reasons.  To that end, three different types of ImmutableData have been defined: Normal, Backup and Sacrificial.  These differ only in how their name is calculated; their contents are identical.

Having three different types bearing different but deterministic names will increase the difficulty of any attack, as it will be three different groups of Vault, which need to be tackled at the same time.

## Supported Use-Cases

From the client perspective, the use-case stays the same: storing or fetching immutable data.  However, the client may need to be aware of the existence of different types.

## Expected Outcome

Replication of the different immutable data types bearing different names, which will be handled by different DataManager groups.

# Detailed design

## Overview

For each type (Normal, Backup and Sacrificial) of a given chunk, the management will be done by the ImmutableDataManagers closest to the chunk's name, and a copy will be stored on two PmidNodes within that close group.

The Normal DMs (DataManagers comprising closest group to the Normal name), forward client Put/Get requests to the Backup and Sacrificial DMs.  This minimises client exposure to the network data types and their management.

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

None identified, other than increased complexity of Vault, Routing and Core codebase.

# Alternatives

1. The SAFE network itself could be free of carrying out any naming calculations and handling based on types if the Client only was aware of these and made requests bearing the different type-dependent names.

1. It is possible that to reduce the code complexity, the Normal DataManager group of an immutable data chunk shall still be DM(normal_name). Then such a group can forward the requests to pmid_nodes closest to normal_name, backup_name, and sacrificial_name. However, due to the fact the pmid_nodes are all already the closest nodes to the data_manager, such a forwarding mechanism may not be able to secure a highly diverse distribution. i.e. some pmid_nodes may hold different type copies at the same time.

# Unresolved Questions

# Future Work

# Appendix
