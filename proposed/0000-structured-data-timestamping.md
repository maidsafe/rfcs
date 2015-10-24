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

```rust
    min_date: i64,
    max_date: i64,
```

## Writing of default values in the existing constructor (StructuredData::new)

The values insure that the behavior of classic SD's remains unchanged with no date validation:

```rust
            min_date : 0i64,
            max_date : i64::max_value(),
```

## Addition of a new constructor

A new constructor (StructuredData::with_dates) with 2 supplementary arguments
specifies a range of dates for the timestamping:

```rust
    /// Constructor with min/max dates.
    /// StructuredData::new cannot simply be called and then dates added because dates must be part of signed data.
    pub fn with_dates(type_tag: u64,
               identifier: ::NameType,
               version: u64,
               data: Vec<u8>,
               current_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               previous_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               signing_key: Option<&::sodiumoxide::crypto::sign::SecretKey>,
               min_date: i64,
               max_date: i64)
               -> Result<StructuredData, ::error::RoutingError> {

        let mut structured_data = StructuredData {
            type_tag: type_tag,
            identifier: identifier,
            data: data,
            previous_owner_keys: previous_owner_keys,
            version: version,
            current_owner_keys: current_owner_keys,
            previous_owner_signatures: vec![],
            min_date : min_date,
            max_date : max_date,
        };

        if let Some(key) = signing_key {
            let _ = try!(structured_data.add_signature(key));
        }
        Ok(structured_data)
    }
```

## Inclusion of the new fields in the replace function

This is done in replace_with_other function.
The date fields must be replaced when an SD is updated.

```rust
        self.min_date = other.min_date;
        self.max_date = other.max_date;
```

## Validation function

It checks that current date is within the specified range.

```rust
    /// Validate date. An error is generated when current date is not in the range [min_date .. max_date]
    pub fn validate_date(&self) -> Result<(), ::error::RoutingError> {
        use time;
        // Don't compute utc time for standard SD (with default values for dates)
        if self.min_date <= 0 && self.max_date == i64::max_value() {
            return Ok(())
        }
        let now_utc = time::now_utc().to_timespec().sec;
        if now_utc < self.min_date || now_utc > self.max_date {
            Err(::error::RoutingError::OutOfRangeDate)
        }
        else {
            Ok(())
        }
    }
```

## Inclusion of the new fields in the signed part of StructuredData

This is done in data_to_sign function.
The aim is to prove that date fields haven't been tampered with (like the rest of the SD).

```rust
        try!(enc.encode(self.min_date.to_string().as_bytes()));
        try!(enc.encode(self.max_date.to_string().as_bytes()));
```

## Two public getter functions

Clients can get the timestamp of a SD by calling these functions.

```rust
    /// Get the min date
    pub fn get_min_date(&self) -> i64 {
        self.min_date
    }

    /// Get the max date
    pub fn get_max_date(&self) -> i64 {
        self.max_date
    }
```

## New error code

OutOfRangeDate is returned by the validation function when current UTC time is not inside the specified range.
The code is defined at 3 places:

RoutingError enumeration:
```rust
    /// Current date is ouside specified range [min_date .. max_date] of structured data
    OutOfRangeDate,
```

Implementation of std::error::Error for RoutingError:
```rust
            RoutingError::OutOfRangeDate => "Current date is outside specified range",
```

Implementation of std::fmt::Display for RoutingError:
```rust
            RoutingError::OutOfRangeDate =>
                ::std::fmt::Display::fmt("Current date is outside specified range", formatter),
```

## Usage from clients

Current client programs will behave the same as before with no date validation.
The new constructor (StructuredData::with_dates) has to be called
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
This one is really simple with only 64 new lines of code (not counting test functions).

# Unresolved questions

## Where to call the date validation function? 

The date validation can be done in the maid managers or the sd managers or both,
I would say it should be done at the same place(s) where the signatures of SD content are verified
(except churn or later events) but I didn't find where this is currently done.

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
