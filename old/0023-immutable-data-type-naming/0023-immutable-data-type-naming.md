# Naming of ImmutableData Types

- Status: implemented
- Type: Enhancement
- Related components: [Routing](https://github.com/maidsafe/routing) and [SAFE Vault](https://github.com/maidsafe/safe_vault)
- Start Date: 26-03-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/111
- Supersedes:
- Superseded by:

## Summary

For security and integrity of data, the design of the SAFE network requires that every chunk of `ImmutableData` be stored at three separate locations.  These locations are derived deterministically from the name of the chunk.  The current implementation uses repeated hashing of the chunk's name to derive the secondary names, which could result in all three locations overlapping.  This RFC proposes a solution to resolve this issue, which also has secondary benefits.

## Motivation

As discussed in the summary, an `ImmutableData` chunk is stored in three locations on the network.  We manipulate the name of the chunk to derive the locations for the copies.  The "Normal" name is the SHA512 hash digest of the contents of the chunk, and this is the primary location.  The secondary locations are defined by the "Backup" name which is the SHA512 digest of the Normal name and by the "Sacrificial" name which is the SHA512 digest of the Backup name.

By using hashing to derive the Backup and Sacrificial names, there is a chance that two or all of the locations will overlap.  In other words, a Vault could end up managing and storing the Normal, Backup and Sacrificial copies of a single chunk.  In an ideal scenario, these locations would have no common Vaults, i.e. no Vault would be responsible for more than one copy of a given chunk.

### Proposed Solution

The proposed solution involves generating the Backup and Sacrificial names by XORing the Normal name with known values.  As the "distance" between two addresses on the SAFE network is calculated by XORing them, and since XOR is a commutative operation, the values chosen to XOR the Normal name with will effectively be the distances from the Normal name to the secondary names.

Since we want the distances between the groups to maximal, we can take the "maximum address" (where this is (2^512) - 1 (i.e. `111..111`)) divided by two as the target distance between groups.  This means we can define the distance between Normal and Backup as 2^511 (i.e. `100..000`) and between Backup and Sacrificial as (2^511) - 1 (i.e. `011..111`).

### Secondary Benefits

The proposed solution has a useful side-effect: for a given name type, both of the other names can be deduced.  This cannot be done for hashing; given a name derived from a hash digest, we cannot calculate the original name, i.e. the hash cannot be reversed.

This is useful since a Vault which is managing a Backup or Sacrificial copy of a chunk would now be able to contact the Normal managers without requiring an actual copy of the chunk.

A further benefit is that XOR is a faster operation than SHA512 hash; in benchmarks, using the proposed solution was between 15% and 20% faster than using the existing implementation.

## Detailed Design

The proposed change would define the following constants and function:

```rust
const NORMAL_TO_BACKUP: [u8; XOR_NAME_LEN] =
    [128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const NORMAL_TO_SACRIFICIAL: [u8; XOR_NAME_LEN] = [255; XOR_NAME_LEN];
const BACKUP_TO_SACRIFICIAL: [u8; XOR_NAME_LEN] =
    [127, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
     255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
     255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
     255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
     255, 255, 255, 255];

fn xor(lhs: &[u8; XOR_NAME_LEN], mut rhs: [u8; XOR_NAME_LEN]) -> XorName {
    for i in 0..XOR_NAME_LEN {
        rhs[i] ^= lhs[i];
    }
    XorName(rhs)
}
```

These would be used in `ImmutableData`'s implementation as follows:

```rust
impl ImmutableData {
    ...
    pub fn name(&self) -> XorName {
        let digest = sha512::hash(&self.value);
        match self.type_tag {
            ImmutableDataType::Normal => XorName(digest.0),
            ImmutableDataType::Backup => xor(&digest.0, NORMAL_TO_BACKUP),
            ImmutableDataType::Sacrificial => xor(&digest.0, NORMAL_TO_SACRIFICIAL),
        }
    }
}
```

It would also allow us to define the following functions to convert between name types:

```rust
pub fn normal_to_backup(name: &XorName) -> XorName {
    xor(&name.0, NORMAL_TO_BACKUP)
}

pub fn backup_to_normal(name: &XorName) -> XorName {
    xor(&name.0, NORMAL_TO_BACKUP)
}

pub fn normal_to_sacrificial(name: &XorName) -> XorName {
    xor(&name.0, NORMAL_TO_SACRIFICIAL)
}

pub fn sacrificial_to_normal(name: &XorName) -> XorName {
    xor(&name.0, NORMAL_TO_SACRIFICIAL)
}

pub fn backup_to_sacrificial(name: &XorName) -> XorName {
    xor(&name.0, BACKUP_TO_SACRIFICIAL)
}

pub fn sacrificial_to_backup(name: &XorName) -> XorName {
    xor(&name.0, BACKUP_TO_SACRIFICIAL)
}
```

A sample implementation including the benchmark tests is available [here](https://gitlab.com/Fraser999/RFC-0023-ImmutableData-Type-Naming).

## Drawbacks

None.

## Alternatives

* Rather than XORing full IDs, simply XOR the first (most significant) byte.
* Continue to use the existing implementation.

## Unresolved Questions

None.
