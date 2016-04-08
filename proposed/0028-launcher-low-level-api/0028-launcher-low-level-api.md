- Feature Name: Exposing Low level APIs for Structured Data and Immutable Data handling
- Type New Feature
- Related components safe_launcher, safe_ffi, safe_core
- Start Date: 06-04-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Proposal for exposing low level APIs from launcher that will allow app devs to
use Structured Data and Immutable data to create their own topologies.

# Motivation

Access to structured data and raw data will be needed for third party applications,
which will allow devs to create their own data structures and also to store raw data.

# Detailed design

NFS provides a confined structure for Directory and File handling, but this
topology might not be the practical data structures that application might need
in real time. Hence, exposing the low level apis which will allow app devs to use
the Structured Data to create and manage their own data structures to build the applications.

To access the low level apis, the applications must request for `LOW_LEVEL_ACCESS`
permission at the time of authorisation with the launcher.

## Structured Data

Structured Data can be used to reference data in the network using an ID and the `tag_type`.
The Id of the structured data is a u8 array of length 64 [u8;64] and the `tag_type` value
can be between the range 10,001 to 2^64.

|tag_type |Operation|Description|
|---------|---------|-----------|
|9| Encrypted| Encrypted Structured Data read and modified only by owner|
|10| Encrypted & Versioned| Version enabled encrypted Structured Data read and modified only by owner|
|11| NotEncypted/Plain |Structured Data for public read but modified only by owner|
|12| NotEncypted/Plain & Versioned|Version enabled Structured Data for public read but modified only by owner|

These tag types will make use of the standard implementation of the Structured Data operations in the
[safe_core](https://github.com/maidsafe/safe_core/tree/master/src/core/structured_data_operations).

At this point, `tag_type between the range 10,001 to (2^64-1) and 9-11` will be permitted by the launcher.
If any specific tag type within the reserved range has to be exposed then it can
also be added later to the permitted range list for the `tag_type` in launcher API.

The Structured Data has size restriction of 100kb. The default implementation in the safe_core
for Structured Data will handle the scenarios even if the size is larger than the allowed size limit.
So the devs using the standard tag types will not have to bother about the size restriction.

But in case, if the devs decide to use more efficient approach than the default implementation,
then they can create a tag_type in the Non Reserved range between (10001 and 2^64-1) and call the APIs. If a
custom tag type is used, then the size restriction should be handled by the application. If the
size is more than the permitted size, then a 413 (Payload too large) HTTP status code will be returned.

If the tag type is in the Custom Range (10001 - 2^64-1) then the data wont be encrypted and will be saved as is.
It becomes the app devs responsibility to encrypt.

### Versioned Structured Data

Versioned Structured Data will have a list of version corresponding to the modifications
that were made. Based on a version Id specific version of the Structured Data can
be retrieved from the network. Unversioned structured data will only return the latest copy.

### Rest API

#### Create

##### Request

###### End point
```
/structuredData
```

###### Method
```
POST
```

###### Body
```javascript
{
  "id": base64 string // [u8;64] array of u8's of length 64 as a base64 String
  "tagType": U64 // Within the permitted range
  "data": base64 // Data that has to be stored as a base64 string    
}
```
|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - 2^64)|
|data| Data to be saved in the Structured Data as base 64 string|

##### Response

###### Header
```
Status: 200 Ok
```

#### List versions

Retrieve the list of versions for the Structured Data. This will work only for version
enabled tag types (10 & 12), else 400 (Bad Request) will be thrown. On success,
available version id list will be returned. 404 (Not found) will be returned if the
structured data for the specified id and tag type is not found.

##### Request

###### End point
```
/structuredData/versions/{id}/{tagType}
```
|Field|Description|
|-----|-----------|
|id|Structured Data Id as base64 string|
|tagType| tagType of the Structured Data (10 or 12)|

###### Method
```
GET
```
##### Response

###### Header
```
Status: 200 Ok
```

###### Response Body
```javascript
[ 'id_v1', 'id_v2' ]
```

#### Get
Retrieves the data held by the Structured Data. When a Structured Data is retrieved,
the header will contain a `sd-version` field with a value.

If the user tries to update an older version of the Structured Data, based on the
`sd-version` value passed while updating can be used to validate the version and a
409 (Conflict) HTTP Status Code will be returned.

In the case of the versioned Structured Data, the `sd-version` will be a base64 string representing the version id.
For the Unversioned Structured Data the `sd-version` will be u64 number which will refer to the [version field in the Structured Data](https://github.com/maidsafe/rfcs/blob/master/implemented/0000-Unified-structured-data/0000-Unified-structured-data.md#structureddata)

##### Request

###### End point
```
/structuredData/{id}/{tagType}
```
|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - 2^64)|
|data| Data to be saved in the Structured Data as base 64 string|

###### Method
```
GET
```
##### Response

###### Header
```
Status: 200 Ok
sd-version: {version-reference}
```

###### Body
JSON as base64 string
```
Data held by the Structured Data as a base64 string
```

#### Get By Version

##### Request

###### End point
```
/structuredData/{id}/{tagType}/{versionId}
```
|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - 2^64)|
|versionId| Version id for which the Structured Data has to be fetched|

###### Method
```
GET
```
##### Response

###### Header
```
status: 200 Ok
sd-version: {version-reference}
```

###### Body
JSON as base64 string
```javascript
Data held by the Structured Data as a base64 string
```


#### Update

Structured data can be updated by passing the `Id, tagType and sd-version` corresponding
to the structured data.

For example, Say Two users using an application request for a structured Data with id ABC,
type tag 9. Assume both the users get the same `sd-version as 5`, which means both have the same copy of the
Structured Data. One user updates the Structured Data for few times and thus the `sd-version` is now at `8`.
When the other user who still has `sd-version 5` when he tries to update, the API must be able to throw a proper
status code describing the conflict in sd-version (409). Based on which the applications can get the latest
Structured Data and updated the same again. If the `sd-version` is not specified in the request,
the latest Structured Data will be updated with the data passed in the update, which might lead of loss of
modifications which might have happened in the mean time.

##### Request

###### End point
```
/structuredData/{id}/{tagType}/{sd-version}?isVersioned=false&isPrivate=false
```
|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - 2^64)|
|sd-version| Optional value - Checks whether the latest version of Structured Data is being updated, if the value is set to true.
Else, will overwrite with the latest data|


###### Method
```
PUT
```

###### Body
```
Data to be saved in the Structured data as base64 String
```

##### Response

###### Header
```
Status: 200 Ok
```

#### Delete

##### Request

###### End point
```
/structuredData/{id}/{tagType}
```

###### Method
```
DELETE
```
##### Response

###### Header
```
Status: 200 Ok
```

### Raw Data

Raw Data can be saved in the network as Immutable Data through the self-encryption process.
When Raw data is written to the network, the data is self-encrypted and split into smaller
chunks and saved as Immutable Data. The self-encryption process returns a DataMap, using which
the actual data can be retrieved. This DataMap is saved to network as raw data and an ID of the
immutable data is obtained which refers to the DataMap.

#### Create

When the raw data is written to the network, the ID of the Immutable Data chunk referring to
the DataMap is returned.

##### Request

##### Endpoint
```
/rawData
```

##### Method
```
POST
```

##### Body
```
data as base64 string
```

#### Response

##### Header
```
status: 200 Ok
```

##### Body
```
ID [u8;64] as bas64 string
```

#### Update

##### Request

###### Endpoint
```
/rawData/{id}?offset=0
```
|Field|Description|
|-----|-----------|
|id| Id referring to the DataMap, obtained after the create operation|
|offset| Optional parameter - if offset is not specified the data is appended to the end of the DataMap|

###### Method
```
PUT
```

###### Body
```javascript
Data as base64 string
```

#### Response

##### Header
```
status: 200 Ok
```

#### Get

##### Request

###### Endpoint
```
/rawData/{id}?offset=0&length=100
```
|Field|Description|
|-----|-----------|
|id| Id obtained after the create operation|
|offset| Optional parameter - if offset is not specified the data is appended to the end of the DataMap|
|length| Optional parameter - if length is not specified the value defaults to the full length|

###### Method
```
GET
```

#### Response

##### Header
```
status: 200 Ok
```

##### Body
```
Data as base64 String
```

# Drawbacks

1. Large file sizes can not be supported. Streaming API will be needed for
supporting large file content.
2. Raw data can not be completely re-written. It can only be updated (partial update)
or appended. Workaround will be to create a new DataMap.  
3. Multi Signature support is not exposed in the API

# Alternatives

None

# Unresolved questions

When Structured Data is saved in the network directly using the low level api,
the launcher wont be able keep an account of the Structured Data saved by the user.
Here the application will only be able to fetch the data from the network and the
users will be able to manage the data only through application and not via launcher.
Should launcher keep track of the Structure Data and immutable Data created by the
user?
