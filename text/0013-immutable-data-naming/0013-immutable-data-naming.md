- Feature Name: ImmutableData naming based on type
- Status: active
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

Having three different types bearing different but deterministic names will increase the difficulty of any attack, as it will probably be three different groups of Vaults, which need to be tackled at the same time.

## Supported Use-Cases

From the client perspective, the use-case stays the same: storing or fetching immutable data.  While clients are aware of the different types (as they're part of the public interface of Routing), they'll only ever have to deal with Normal chunks, both when putting and getting ImmutableData.

## Expected Outcome

Replication of the different immutable data types bearing different names, which will be handled by the DataManager group closest to the given chunk's name.  Vaults will not be required to do any calculations on whether it should hold a particular chunk - Routing will ensure that only appropriate chunks are passed to Vaults.

# Detailed design

## Overview

For each type (Normal, Backup and Sacrificial) of a given chunk, the management will be done by the ImmutableDataManagers closest to the chunk's name, and a copy will be stored on two PmidNodes within that close group.

The Normal DMs (DataManagers comprising closest group to the Normal name), forward client Put requests to the Backup and Sacrificial DMs, and forward Get requests to the Backup and Sacrificial DMs only if required (e.g. in case of heavy churn where the Normal copies are unavailable).  This minimises client exposure to the network data types and their management.

## Implementation Details

### Put

MMs (MaidManagers) should respond with failure for any Put or Get requests from Clients for Backup or Sacrificial types.

DMs handling a Put request for a Normal chunk from MM should:

* Send Put to the two PM (PmidManager) groups closest to the data name
* Construct a Backup and a Sacrificial copy of the chunk and send Put requests for these to the DM groups for each

DMs handling a Put request for a Backup or Sacrificial chunk from DM should:

* Send Put to the two PM (PmidManager) groups closest to the data name

PNs (PmidNodes) without enough free space to handle a Put request for any type chunk from PMs should respond with failure.

DMs handling a Put failure for Normal or Backup chunks should send Delete requests for Sacrificial chunks which the same PN holds to the PM group.  Once the responses have all arrived, the initial Put request should be retried.

If a PN is ultimately unable to store a chunk even after having been instructed by the DMs to delete all Sacrificial chunks, it should be marked as "bad" by the DMs and not retried as a holder for that chunk.  The Put process stops at a DM group once a copy is held on two different PNs from the close group for that chunk, or once all Vaults in the close group have been attempted and failed.

### Get

DMs handling a Get request for a Normal chunk from client should send a Get request to each "good" PN concurrently.  As soon as a successful response arrives, the Client's request should be responded to.

Any failure responses should cause that PN to be marked as "bad" and a replacement copy should be Put to a new PN.  If both potential PNs have failed to provide the chunk, the DMs will need to send Get requests to the DMs for one of the other types of the same chunk.  Normal will request from Backup, and if this fails from Sacrificial.  Backup will request from Normal, then Sacrificial.  Sacrificial will request from Normal, then Backup.

When receiving a successful Get response for a Backup or Sacrificial chunk, if this is to be sent as a response to a client it must be converted into a Normal chunk.

It is expected that three groups should never all fail for any given chunk.

### Churn

If a current PN ceases to be a holder for a chunk due to churn (either it has disconnected or it has been pushed out of the close group for that chunk due to other Vaults joining the network) then it should be removed from the list of holders and a replacement copy should be Put to a new PN.  This will likely involve following the Get procedure first.

## Planned Work

1. MaidManager
    1. Block attempts to Put or Get Backup or Sacrificial chunks

1. DataManager
    1. Make accounts aware of the type of chunk
    1. Put Backup and Sacrificial copies when handling a client Put
    1. Handle Puts from other DMs for Backup and Sacrificial chunks
    1. Handle failure to Get by trying to Get a different type of the same chunk
    1. Handle successful Get response of non-Normal type where this needs converted to a Normal chunk to satisfy a client Get request
    1. Handle failure to Put on PN by Deleting Sacrificial chunks on that Vault and retrying Put

1. PmidManager
    1. Handle Delete requests from DMs for Sacrificial chunks only

1. PmidNode
    1. Handle Delete requests from PMs for Sacrificial chunks only

# Drawbacks

Increased complexity of Vault codebase.

# Alternatives

1. The SAFE network itself could be free of carrying out any naming calculations and handling based on types if the Client only was aware of these and made requests bearing the different type-dependent names.

1. It is possible that to reduce the code complexity, the Normal DataManager group of an immutable data chunk shall still be DM(normal_name). Then such a group can forward the requests to pmid_nodes closest to normal_name, backup_name, and sacrificial_name. However, due to the fact the pmid_nodes are all already the closest nodes to the data_manager, such a forwarding mechanism may not be able to secure a highly diverse distribution. i.e. some pmid_nodes may hold different type copies at the same time.

# Unresolved Questions

# Future Work

# Appendix
