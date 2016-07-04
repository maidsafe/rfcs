# Append by all Structured Data Type

- Type New feature
- Status: proposed
- Related components safe_launcher, safe_ffi, safe_core, safe_vault
- Start Date: 04-04-2016
- Discussion: https://github.com/maidsafe/rfcs/issues/113
- Supersedes:
- Superseded by:

## Summary

The proposal details on how a new Structured Data Type which can be appended by all
can enable dynamic data handling.


## Motivation

Currently the SAFE Network supports static data storage and retrieval. This limits
potential application developers to building only static applications and in real world the majority
of applications are dynamic in nature. Enabling dynamic data handling will allow application
developers to create and manage their own data structures using Structured Data and Immutable Data
from the SAFE Network.

## Detailed Design

Structured data at present can only be modified by the owner. If it has multiple owners,
then at least (n/2) + 1 owners must sign the Structured Data for any update or delete.

This design does not scale when it comes to handling dynamic data. For dynamic data
handling the Structured data should be modifiable by the users of the application.

Thus, proposing a new behaviour for the Structured Data with `tag_type - 8`. The Structured
Data with `tag_type 8` can be appended by anyone. The vaults wouldn't be checking the
ownership but instead it would allow anyone to modify the content of the Structured Data.
However, the vaults would validate the ownership of the Structured Data for a delete operation.

Combining the append by anyone Structured Data type and Immutable Data, the doors for handling
dynamic data can be opened up in the SAFE Network.

Low level APIs for directly working with Structured Data and Immutable Data must be exposed
from the Launcher. The applications must request `LOW_LEVEL_API_ACCESS` permission at the time of
authorisation for invoking the low level APIs for Structured Data and
Immutable Data access.

### A Practical Use Case - Simple Forum Application

The appendable Structured Data and Immutable Data can be used to create a dynamic
application like a `Forum`.

#### Hosting the Application - Same as we do for the static websites

- The admin of the application creates a public name and a service name. Let us consider,
that the service name is `forum` and the public name is `maidsafe`, making it accessible
from a browser as `http://forum.maidsafe.net`
- The application is hosted on the SAFE Network just like a website is hosted by mapping
the public folder (source of the application) with the service.

#### Initial Configuration - Creating Root / Master Appendable Structured Data

- After making the application public, the admin registers themselves as the owner / admin of the
application.
- Assume that the admin will have to configure the list of `tags` like `updates, development,
marketing` during the initial set up.
- The application will create a root / master Structured Data with `tag_type 8`. The actual data
held by the Structured Data can be any data structure that would fit the needs of the application.
In this use case we can consider a simple JSON object that would be stored in the root
Structured Data.
```
{
	"tags": ["updates", "development", "marketing"],
 	"posts": []
 }
```
- When the root / master Structured Data is created, the user who creates the Structured Data
will eventually become the owner. At the time the Structured Data is created, the owner field will
contain the public key of the user.
- Thus, when the admin configures the application and saves the metadata like tags, the root
Structured Data can be created which will make admin the owner of the application.
- The root Structure Data plays a very important role as it is the single source from which the
application can get the information while loading. Thus a deterministic approach must be in place
for identifying the root Structured Data.
- The hash of service name and public name can be used as the ID for the root / master Structured
Data.

##### Summing it up

When the admin configures the application for the first time. The application will create a root
Structured Data with `tag_type as 8` and with id `SHA512(service name + public name)`.
The data part of the Structured Data can be anything that would fit the needs of the application
like, `csv, toml, json` etc. In this use case we will be using a JSON object.

##### Application looking up for the master Structured Data on start

When a user reaches the end point (forum.maidsafe.safenet), the application will be able to lookup
the Structured Data with the id `SHA512(service name + public name)`. Once the data is retrieved,
based on the JSON object the rest of the data needed by the application can be fetched.

##### When a new thread is created

Say user ABC has set up the forum and they become the admin. Now, let consider user XYZ wants to
create a new thread. When a new thread is created, the data related to the thread is stored in the
network as a JSON object in the form of Immutable Data chunks. The DataMap of the thread will be
added to the root / master Structured Data.

###### Detailing the steps in sequence

- User XYZ logs in to the Launcher with their own credentials and goes to `forum.maidsafe.safenet`
- Application will look up for the master / root Structured Data by hashing the service and long
name. The Structured Data when fetched from the network will have the JSON object to fetch the list
of posts from the network and load them.
- When a user creates a new thread, the thread is stored as a JSON object in the network using
the low level APIs for creating Immutable Data exposed by the safe_launcher. The thread is converted
to a JSON representation and stored within the network. The DataMap received after the data is
written to the network is added to the appendable root Structured Data and saved within the network.
New DataMap is added to the posts list in the root Structured Data.
```
{
	"tags": ["updates", "development", "marketing"],
 	"posts": [ datamap_thread_1_v1 ]
 }
```
- The JSON representation of the thread can be,
```
{
 	"title": "hello world",
 	"createdTime": "timeinUTC",
  "createdBy": "ABC", // public name of the user who created the thread
  "tags": [],
  "body': "Actual content of the thread goes here",
  "replies": []
}
```
###### When a reply is made to a post

When other users reply to the thread, a new DataMap can be generated and updated in the root /
master Structured Data.
```
{
	"tags": ["updates", "development", "marketing"],
 	"posts": [datamap_thread_1_v2]
 }
 ```
Here v2 represents the new DataMap.


## Drawbacks

Since the root / master Structured Data is generated / looked up in a deterministic manner
and moreover it can also be appended by anyone, this makes it easier for an attacker to corrupt
the data. A simple CLI tool could be created using safe_core and the Structured Data can be modified
or even cleared.
DNS also uses a deterministic approach, but the Structured Data is secure as it can be modified only
by the owner. But in this appendable Structured Data, it becomes easier to corrupt the data.


## Alternatives

None

## Unresolved Questions

Concurrency Issue: When two users are replying to a same thread at the same time.
Only the last updated DataMap will be reflected. This will result in the loss of the
reply made by the first user. This can not be handled by the Structured Data versions because in
this case we are using only one master Structured Data for the application and the number of
concurrent changes could be very high. Even a `like` or a reply would force the entire
DataMap to be generated and also the root Structured Data to be updated.
