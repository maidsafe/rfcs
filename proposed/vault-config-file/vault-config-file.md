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


## Expected Outcome

A separated config file co-existing with the vault exectuable. This will only be accessed during the intialization procedure. Vault executable no longer using any fixed assumed value for chunk_store initialization.

# Detailed design

## Overview

The config file shall contains following items:
	1. wallet_address : the associated address that shall get rewarded for the service provided, in a format of hex code
	1. max_space_pmid_node : the max disk space allocated for pmid_node, measured in Byte
	1. max_space_sd : the max disk space allocated for structured_data manager, measured in Byte

A sample config file may looks like (1GB for pmid_node and 100MB for sd_manager):
```rust
{
	wallet_address : 245df3245df3245df3245df300000000000000001cc0dd1cc0dd1cc0dd1cc0dd
	max_space_pmid_node : 1073741824
	max_space_sd : 104857600
}
```

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

