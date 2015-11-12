- Feature Name: ImmutableData naming base on type
- Type: New Feature
- Related Components: [safe_vault](https://github.com/maidsafe/safe_vault), [launcher](https://github.com/maidsafe/safe_launcher)
- Start Date: 10-11-2015
- RFC PR:
- Issue number: Proposed -

# Summary

This RFC outlines the config file that being required by the vault to initialise personas.

# Motivation

## Rationale

During the disscussion [safecoin implementation](https://github.com/maidsafe/rfcs/issues/61) of [RFC SafeCoin Implementation](https://github.com/maidsafe/rfcs/blob/master/proposed/0012-safecoin-implementation/0012-safecoin-implementation.md), it is being assumed that a pmid_node persona needs to be aware of it's owner's wallet address.

In addition to this, when chunk_store being intialized (currently being used by PmidNode and StructruedDataManager personas only), a configurable max usable disk space needs to be specified (currently a fixed value of 1GB is being assumed to be used as default). 

To resolved the above issues, a separate config file for vault is proposed to allow user configurable parameters can be loaded and keeps retained across session (restarting of vault).

## Supported Use-Cases

1. User can specify the storage space for pmid_node and sd_manager

1. User can specify the account to be rewarded for the service provided to SAFE network

1. In case of restart, the configuration keeps the same

1. In case of the re-installation of Vault executable, the config file shall not be changed.


## Expected Outcome

A separated config file co-existing with the vault exectuable. This will only be accessed during the intialization procedure. Vault executable no longer using any fixed assumed value for chunk_store initialization.

# Detailed design

## Overview

The config file shall contains following items:
	1. wallet_address : the associated address that shall get rewarded for the service provided, in a format of hex code
	1. max_space : the max disk space allocated for this vault node, measured in MBytes

It needs to be mentioned that there is an internal distribution ratio between pmid_node (for immutable_data) and sd_manager (for structured_data). This RFC is proposing such ratio to be 3:1. i.e. if 100MB has been set for a vault, 75MB will be used by pmid_node and 25MB will be used by sd_manager.

A sample config file may looks like (1GB for pmid_node and 100MB for sd_manager):
```rust
{
	wallet_address : 245df3245df3245df3245df300000000000000001cc0dd1cc0dd1cc0dd1cc0dd
	max_space : 100
}
```

Following rules shall also be applied:
1. A wallet address must be presented to start up a vault
1. A default vault of 1000MB for the max storage space will be used if it is not set.

## Implementation Details


## Planned Work

1. Vault
    1. Config file handler
    1. PmidNode code update
    1. Structured Data code update

1. Launcher
    1. includes the config file in the installation package
    1. An instruction to view and modify the config file



# Drawbacks

None identified, other than increased complexity of Vault and Launcher.

# Alternatives

1. It is possible to use command line interaction or passing the listed configurations as program parameters. However, this may cause some issue to the user experience, especially when vault executable needs to be restarted.


# Unresolved Questions



# Future Work



# Appendix

