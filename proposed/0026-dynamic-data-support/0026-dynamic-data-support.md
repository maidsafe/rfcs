- Feature Name: Ability to Support Dynamic Data Handling
- Type enhancement
- Related components safe_launcher, safe_ffi, safe_core, safe_vault
- Start Date: 04-04-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

The proposal details on how dynamic data can be supported in the SAFE Network.

# Motivation

Currently the SAFE Network supports static data storage and retrieval. This limits
the application developers to build only static applications. In real world, most of
the applications are dynamic in nature. Enabling dynamic data handling will allow
application developers to create and manage their own data structures using the
Structured Data and Immutable Data from the SAFE Network.

# Detailed design

Structured data at present can only be modified by the owner. If it has multiple owners,
then at least (n/2) + 1 owners must sign the structured data for any update or delete.

This design does not scale when it comes to handling dynamic data. For dynamic data
handling the Structured data should be modifiable by the users of the application.

Thus, proposing a new behaviour for the structured data with `tag_type - 8`. The Structured
data with `tag_type 8` can be appendable by anyone. The vaults wouldn't be checking the
ownership but instead it would allow anyone to modify the content of the structured data.
However, the vaults would validate the ownership of the structured data for a delete operation.

Combining the appendable structured data and immutable data, the doors for handling dynamic
data can be opened up in the SAFE Network.

Low level APIs for directly working with Structured Data and Immutable Data must be exposed
from the launcher. The applications must request for `LOW_LEVEL_API_ACCESS` permission at
the time of authorization for invoking the Low level APIs for structured data and
immutable data access.

## A Practical Use Case - Simple Forum application

The appendable structured data and immutable data can be used to create a dynamic
application like a `Forum`.

### Hosting the application - Same as we do for the static websites
- The Admin of the application creates a public name and a service for the same. Let us consider,
that the service name is `forum` and the public name is `maidsafe`, making it accessible
from browser as `http://forum.maidsafe.net`
- The application is hosted in the SAFE Network just like a website is hosted by mapping
the public folder (Source of the application) with the service.

### Initial Configuration - Creating Root/Master Appendable Structured Data
- After making the application public, the admin registers himself as the owner/admin of the application.
- Assume that the admin will have to configure the list of `tags` like `updates, development, marketing`
during the initial set up.
- The application will create a root/master Structured Data with `tag_type 8`. The actual data held
by the structured data can be any data structure that would be fitting the needs of
the application. In this use case we can consider a simple JSON object that would be stored
in the root structured data.
```
{
	"tags": ["updates", "development", "marketing"],
 	"posts": []
 }
```
- When the root/master structured data is created, the user who creates the structured data
will eventually become the owner. At the time of creation of the structured data, the owner field will
contain the public key of the user.
- Thus, when the admin configures the application and saves the metadata like tags, the root
Structured data can be created which will make admin as the owner of the application.
- The root structure data plays a very important role as it is the single source from which the
application can get the information while loading. Thus a deterministic approach must be in place
for identifying the root structured data.
- The hash of service name and public name can be used as the ID for the root/master structured data.

#### Summing it up
When the admin configures the application for the first time. The application will create a root
structured data with `tag_type as 8` and with id `SHA512(service name + public name)`.
The data part of the structured data can be anything that would fit the needs of the application like,
`csv, toml, json` etc. In this use case we would be using a JSON object.

#### Application looking up for the master Structured data on start
When a user reaches the end point (forum.maidsafe.safenet), the application will be able to lookup
for the structured data with the id `SHA512(service name + public name)`. Once the data
is fetched, based on the JSON object rest of the data needed by the application can be fetched.

#### When a new thread is created
Say user ABC has set up the forum and he becomes the admin. Now, let consider user XYZ wants to create
a new thread. When a new thread is created, the data related to the thread is stored in the network
as a JSON object in the form of immutable data chunks. The DataMap of the thread will be added to the
root/master Structured data.

##### Detailing the steps in sequence
- User XYZ logs in to the launcher with his own credentials and goes to `forum.maidsafe.safenet`
- Application will look up for the master/root structured data by Hashing the service and long name.
The structured data when fetched from the network will have the JSON object to fetch the list of posts
from the network and load the same.
- When a user creates a new thread, the thread is stored as a JSON Object in the network using
the low level APIs for creating immutable data exposed by the safe_launcher. The thread is converted to a
JSON representation and stored in the network. The DataMap received after the data is written to the network
 is added to the appendable root structured data and saved in the network. New DataMap is added to the
 posts list in the root structured data.
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
##### When a reply is made to a post
When other users reply to the thread, a new DataMap can be generated and updated
in the root/master structure data.
```
{
	"tags": ["updates", "development", "marketing"],
 	"posts": [datamap_thread_1_v2]
 }
 ```
Here v2 represents the new DataMap.


# Drawbacks

Since the root/master Structured Data is generated/looked up in a deterministic manner
and moreover it is also appendable for anyone, makes it easier for an attacker to corrupt
the data. A simple CLI tool can be created using safe_core and the structured data can be modified or even cleared.
DNS also uses a deterministic approach, but the structured data is secure as it can be modified only by the owner.
But in this appendable structured data, it becomes easier to corrupt the data.


# Alternatives

None

# Unresolved questions

Concurrency Issue. When two users are replying to a same thread at the same time.
Only the last updated DataMap will be reflected. This will result in the loss of the
reply made by the first user. This can not be handled by the structured data versions because in this
case we are using only one master structured data for the application and the number of concurrent
changes can be very high. Even for a `like` or for a reply would enforce the entire DataMap
to be generated and also the root structured data to be updated.
