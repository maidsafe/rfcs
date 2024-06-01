- Feature Name: structured-data-timestamping
- Type: new feature
- Related components: routing
- Start Date: 11-10-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Timestamping is used to prove the existence of certain data
(e.g. contracts, inventions description, artwork, ...)
at a certain date without the possibility that the owner can backdate the timestamp.
The aim of the RFC is to allow optional timestamping of Structured Data (SD)
implemented in a way that doesn’t need a complex time synchronization mechanism in the safe network.

# Motivation

The bitcoin blockchain can prove that a document existed at a certain date
by storing the hash of a document but cannot store the document itself.
The safe network does the contrary: it can store the document but cannot prove the date of storage.
The aim of the proposal is to fill this gap so that the safe network is able to store both:

- the document itself
- an approximate date of storage of the document in network

Merging the best of both worlds allows numerous use cases:

- audit systems
- medical records
- supply chain management
- voting systems
- property titles
- legal applications
- financial systems
- ...

Of course all these are fulfilled without a central authority.

# Detailed design

The implementation is simple because it doesn't need any synchronization between the nodes:

- no NTP servers
- no exchange of dates

This is possible because most devices are already approximatively synchronized in the range of a few minutes
and timestamping of documents doesn't need to be precise.

Concretely, to handle the date variability among the nodes, the client defines a date range in the SD.
During the validation consensus a manager has only to check that its own date is inside this range.

## Addition of a date range in StructuredData

It is stored in 2 new fields. The i64 type represents the sec part of a struct Timespec
generated from an UTC time (a number of seconds since the beginning of the epoch).

These fields are embedded in an enum that indicates whether or not the SD is a ledger item.
This enum specify the delete behaviour so that a ledger item cannot be recreated with the
same name and the same version but a different content.

```rust
/// Indicate if SD is a ledger item.
///
/// A ledger item is a SD that is not erased from the network when it is deleted.
/// Instead a new version is stored with an empty payload and its version is incremented by 1
/// (equivalent to a Post command).
/// Optionally, a timestamp can be specified.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone, RustcDecodable, RustcEncodable)]
pub enum LedgerItem {
    /// SD is not a ledger item
    None,
    /// SD is a ledger item but has no timestamp
    NoTimestamp,
    /// SD is a ledger item and has a timestamp
    WithTimestamp {
        /// Lower bond of timestamp
        min_date: i64,
        /// Upper bond of timestamp
        max_date: i64,
    }
}
```

## Writing of a default value in the existing constructor (StructuredData::new)

This value insures that the behavior of classic SD's remains unchanged (they are not ledger items
and have no date validation).

```rust
            ledger_item: LedgerItem::None,
```

## Addition of a new constructor

A new constructor (StructuredData::with_ledger_item) with a supplementary argument that specifies
if SD is ledger item with an optional range of dates for timestamping:

```rust
    /// Constructor with ledger item.
    /// StructuredData::new cannot simply be called and then ledger item added because ledger item must be part of signed data.
    pub fn with_ledger_item(type_tag: u64,
               identifier: XorName,
               version: u64,
               data: Vec<u8>,
               current_owner_keys: Vec<PublicKey>,
               previous_owner_keys: Vec<PublicKey>,
               signing_key: Option<&SecretKey>,
               ledger_item: LedgerItem)
               -> Result<StructuredData, ::error::RoutingError> {

        let mut structured_data = StructuredData {
            type_tag: type_tag,
            identifier: identifier,
            data: data,
            previous_owner_keys: previous_owner_keys,
            version: version,
            current_owner_keys: current_owner_keys,
            previous_owner_signatures: vec![],
            ledger_item: ledger_item,
        };

        if let Some(key) = signing_key {
            let _ = try!(structured_data.add_signature(key));
        }
        Ok(structured_data)
    }
```

## Inclusion of the new field in the replace function

This is done in replace_with_other function.
The ledger item must be replaced when an SD is updated.

```rust
        self.ledger_item = other.ledger_item;
```

## Validation function

It checks that current date is within the specified range.

```rust
    /// Validate date. An error is generated when current date is not in the range [min_date .. max_date]
    pub fn validate_date(&self) -> Result<(), ::error::RoutingError> {
        use time;
        match self.ledger_item {
            LedgerItem::WithTimestamp { min_date, max_date } => {
                let now_utc = time::now_utc().to_timespec().sec;
                if now_utc < min_date || now_utc > max_date {
                    Err(::error::RoutingError::OutOfRangeDate)
                }
                else {
                    Ok(())
                }
            },
            // Don't validate date of non ledger item or ledger item without dates
            _ => Ok(())
        }
    }
```

## Validation against successor

To control that a ledger item is not replaced by a non ledger item the following code is added to
validate_self_against_successor function:

```rust
        // Replacing a ledger item by a non ledger item is forbidden because this would allow complete deletion
        // and then recreation of SD.
        if other.ledger_item == LedgerItem::None && self.ledger_item != LedgerItem::None {
            return Err(::error::RoutingError::LedgerItemReplaced);
        }
```

## Inclusion of the new field in the signed part of StructuredData

This is done in data_to_sign function.
The aim is to prove that ledger item field hasn't been tampered with (like the rest of the SD).

```rust
            ledger_item: self.ledger_item.clone(),
```

## Expose LedgerItem type to clients

To facilitate usage of leger items, this type must be exposed at the root level of routing namespace in lib.rs:

```rust
pub use structured_data::{MAX_STRUCTURED_DATA_SIZE_IN_BYTES, StructuredData, LedgerItem};
```

## Public getter functions

Clients can get the timestamp of a SD by calling these functions. If the SD is not a legder item or is a ledger item without timestamp,
their return values defines a timestamp with a range from beginning of EPOCH to 2^63 - 1 seconds after it
(which means no timestamp concretely).


```rust
    /// Get timestamp lower bound
    pub fn get_min_date(&self) -> i64 {
        match self.ledger_item {
            LedgerItem::WithTimestamp { min_date, .. } => {
                min_date
            }
            // No timestamp
            _ => 0i64
        }
    }

    /// Get timestamp upper bound
    pub fn get_max_date(&self) -> i64 {
        match self.ledger_item {
            LedgerItem::WithTimestamp { max_date, .. } => {
                max_date
            }
            // No timestamp
            _ => i64::max_value()
        }
    }
```

Clients can also test is a SD is a ledger item:

```rust
    /// Test if SD is a ledger item
    pub fn is_ledger_item(&self) -> bool {
        self.ledger_item != LedgerItem::None
    }
```

## New error codes in RoutingError enumeration

OutOfRangeDate is returned by the validation function when current UTC time is not inside the specified range.

```rust
    /// Current date is ouside specified range [min_date .. max_date] of structured data
    OutOfRangeDate,
```

LedgerItemReplaced is returned by validate_self_against_successor function when an attempt is made to change
a ledger item by a non ledger item. This is forbidden because this would allow real deletion and then recreation
of SD.

```rust
    /// Ledger item replaced by non ledger item
    LedgerItemReplaced,
```

## Usage from clients

Current client programs will behave the same as before with no date validation.
The new constructor (StructuredData::with_ledger_item) has to be called
to activate the timestamping functionality.

To maximize the chances that its request is valid the client can give a very large range
like current date ± one hour. This is not a problem because most of the times
people don't need a precise dating.

If it really needs a precise dating, the client can also try different names for the SD
until it is handled by a group with a majority of more precise nodes (28/32 consensus majority).

## Usage from vaults

The validation function (validate_date) must be called from the managers during both PUT and POST operations.
By revalidating during POST, an owner is forced to update the date fields when he modifies a SD
(otherwise the managers wouldn't validate the original dates because time has passed since initial PUT).

That way, an owner cannot substitute the previous data content by a new one
and keep the same previous timestamps, which would backdate the new data content.
So, backdating a document is not possible.

# Drawbacks

Date validation cannot be done at later stages, like for example during a churn event.
This is mitigated by the fact that signatures
can be checked again at these stages, which will prove that the date fields haven't been tampered with.
This is the same way that others fields are revalidated (like the data itself).

# Alternatives

Other designs with time synchronization between nodes are a lot of more complex to implement.
This one is really simple with about 100 new lines of code (not counting test functions).

# Solved questions

## Where to call the date validation function?

The date validation can be done in the maid managers or the data managers or both,

If done in data managers the following pieces of code are to be added in safe_vault/src/personas/data_manager.rs:

- In function handle_put:

```rust
        if let Data::Structured(ref data) = data {
            // Validate min/max dates
            if let Err(_) = data.validate_date() {
                let error = MutationError::InvalidSuccessor;
                let external_error_indicator = try!(serialisation::serialise(&error));
                trace!("DM sending PutFailure for data {:?}, invalid timestamp.",
                       data_id);
                let _ = self.routing_node
                    .send_put_failure(dst, src, data_id, external_error_indicator, message_id);
                return Err(From::from(error));
            }
        }
```
- In function handle_post:

```rust
        // Validate min/max dates
        if let Err(_) = new_data.validate_date() {
            let error = MutationError::InvalidSuccessor;
            let external_error_indicator = try!(serialisation::serialise(&error));
            trace!("DM sending post_failure for data {:?}, invalid timestamp.",
                   data_id);
            let _ = self.routing_node
                .send_put_failure(dst, src, data_id, external_error_indicator, message_id);
            return Err(From::from(error));
        }
```

## Where to manage deletion of ledger items?

This is to be done in function handle_delete of safe_vault/src/personas/data_manager.rs. The following code
should be executed when validation against successor is OK:

```rust
                if !data.is_ledger_item() {
                    // Not a ledger item => erase SD from chunk store
                    if let Ok(()) = self.chunk_store.delete(&data_id) {
                        self.count_removed_data(&data_id);
                        trace!("DM deleted {:?}", data.identifier());
                        info!("{:?}", self);
                        let _ = self.routing_node.send_delete_success(dst, src, data_id, message_id);
                        // TODO: Send a refresh message.
                        return Ok(());
                    }
                } else {
                    // Ledger item => check that data part is empty and store empty SD in chunk store
                    if new_data.payload_size() == 0 && new_data.validate_date().is_ok() {
                        let version = new_data.get_version();
                        if let Err(error) = self.chunk_store.put(&data_id, &Data::Structured(new_data)) {
                            trace!("DM sending delete_failure for: {:?} with {:?} - {:?}",
                                   data_id,
                                   message_id,
                                   error);
                            let mutation_error =
                                MutationError::NetworkOther(format!("Failed to store chunk: {:?}", error));
                            let post_error = try!(serialisation::serialise(&mutation_error));
                            return Ok(try!(self.routing_node
                                .send_delete_failure(dst, src, data_id, post_error, message_id)));
                        }
                        trace!("DM updated for: {:?}", data_id);
                        let _ = self.routing_node.send_delete_success(dst, src, data_id, message_id);
                        let data_list = vec![(data_id, version)];
                        let _ = self.send_refresh(Authority::NaeManager(data_id.name()), data_list);
                        return Ok(())
                    }
                }
```


# Unresolved question

## Can it work if date variability among the nodes is too important?

This solution only works if a consensus majority of nodes (28/32) have a reasonable difference with actual time.

This can be checked by asking the community to connect to a site like http://time.is/
and verify that the gap of their devices is less than 10 minutes
(as an example, to get the same precision as the bitcoin blockchain).

I personally checked at work (12 stations: servers and desktops) and at home
(9 devices: desktops, laptops, tablets, smartphones)
and they are all in the range of actual time ± one minute, which is more than enough.

Even if there is a too large minority of unsynchronized devices (more than 4/32)
then statistical variance will create groups with less than 4/32 unsynchronized nodes.
When such a group receive a request with a narrow date range, the nodes
which don't validate the request can be deranked.
This deranking mechanism will progressively "purify" the network and the nodes will converge
to more uniformity in UTC time.

But I don't think this needs to happen because the internet network is already sufficiently synchronized
for the needs of document timestamping.
