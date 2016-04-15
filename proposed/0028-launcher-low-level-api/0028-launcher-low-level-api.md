- Feature Name: Exposing Low level APIs for Structured Data and Immutable Data handling
- Type New Feature
- Related components safe_launcher, safe_ffi, safe_core
- Start Date: 06-04-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Proposal for exposing low level APIs from Launcher that will allow app devs to
use Structured Data and Immutable data to create their own topologies.

# Motivation

Access to Structured Data and raw data will be needed for third party applications,
which will allow devs to create their own data structures and also to store raw data.

# Detailed Design

NFS provides a confined structure for directory and file handling, but this
topology may not be the practical data structures that applications need
in real time. Hence, exposing the low level APIs will allow app devs to use
Structured Data to create and manage their own data structures to build applications.

To access the low level APIs, the application must request `LOW_LEVEL_ACCESS`
permission at the time of authorisation with the Launcher.

**Only Authorised requests can access the low level APIs.**

1. Authorised requests should contain the token in the Authorization header.
2. The query parameters and the request body must be encrypted using the symmetric key.
3. Response body will be encrypted using the symmetric key.

## Structured Data

Structured Data can be used to reference data in the network using an ID and the `tag_type`.
The ID of the Structured Data is a u8 array of length 64 [u8;64] and the `tag_type` value
can be between the range 10,001 to (2^64 - 1).

|tag_type |Operation|Description|
|---------|---------|-----------|
|9| Encrypted| Encrypted Structured Data read and modified only by owner.|
|10| Encrypted & Versioned| Version enabled encrypted Structured Data read and modified only by owner.|
|11| NotEncypted/Plain |Structured Data for public read but modified only by owner.|
|12| NotEncypted/Plain & Versioned|Version enabled Structured Data for public read but modified only by owner.|

These tag types will make use of the standard implementation of the Structured Data operations in the
[safe_core](https://github.com/maidsafe/safe_core/tree/master/src/core/structured_data_operations).

At this point, `tag_type between the range 10,001 to (2^64-1) and 9-11` will be permitted by the Launcher.
If any specific tag type within the reserved range has to be exposed then it can
also be added later to the permitted range list for the `tag_type` in the Launcher API.

The Structured Data has a size restriction of 100KB. The default implementation in the safe_core
for Structured Data will handle the scenarios even if the size is larger than the allowed size
limit. So the devs using the standard tag types will not have to bother about the size restriction.

If the devs decide to use a different approach other than the default implementation,
then they can create a tag_type in the non reserved range between (10001 and 2^64-1) and call the APIs. If a custom tag type is used, then the size restriction should be handled by the application. If the size is more than the permitted size, then a 413 (payload too large) HTTP status code will be returned.
Moreover, if the tag type is within the custom range (10001 - 2^64-1) then the data will be saved as is.
It becomes the app devs responsibility to encrypt, verify size, etc.

### Versioned Structured Data

Versioned Structured Data will have a list of versions corresponding to the modifications
that have been made. Based on a version ID a specific version of the Structured Data can
be retrieved from the network. Unversioned Structured Data will only return the latest copy.

### Rest API

#### Create

Structured Data api will return the version ID on success for tag_types 10 & 12 (Versioned Structure Data).
The version Id will be base64 string representing [u8;64]

##### Request

###### End point
```
/structuredData/{id}/{tagType}
```

|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string.|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - (2^64 - 1)).|

###### Method
```
POST
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

###### Body
```javascript
Data that has to be stored as a base64 string  
```

##### Response

###### Header
```
Status: 200 Ok
```

###### Body
Only for Versioned Structure Data
```
Version Id as base64 string
```

#### List versions

Retrieve the list of versions for the Structured Data. This will work only for version
enabled tag types (10 & 12), otherwise a 400 (Bad Request) will be thrown. On success,
an available version ID list will be returned. 404 (Not Found) will be returned if the
Structured Data for the specified id and tag type is not found.

##### Request

###### End point
```
/structuredData/versions/{id}/{tagType}
```
|Field|Description|
|-----|-----------|
|id|Structured Data Id as base64 string.|
|tagType| tagType of the Structured Data (10 or 12).|

###### Method
```
GET
```

###### Headers
```
Authorization: Bearer <TOKEN>
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
the header will contain a `SD-Version` field with a value.

If the user tries to update an older version of the Structured Data - based upon the
`SD-Version` value passed while updating will be used to validate the version and a
409 (Conflict) HTTP Status Code will be returned.

In the case of the versioned Structured Data, the `SD-Version` will be a base64 string representing the version id.
For the Unversioned Structured Data the `SD-Version` will be a u64 number which will refer to the [version field in the Structured Data](https://github.com/maidsafe/rfcs/blob/master/implemented/0000-Unified-structured-data/0000-Unified-structured-data.md#structureddata)

The response header will also have a `Owner` field, which will hold the owners public key as a
base64 string.

##### Request

###### End point
```
/structuredData/{id}/{tagType}
```
|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string.|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - (2^64 - 1)).|

###### Method
```
GET
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

##### Response

###### Header
```
Status: 200 Ok
SD-Version: {version-reference}
Owner: base64 string - representing public key of user
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
|id   | u8 array of length 64 as a base64 string.|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - (2^64 - 1)).|
|versionId| Version ID for which the Structured Data has to be fetched.|

###### Method
```
GET
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

##### Response

###### Header
```
status: 200 Ok
SD-Version: {version-reference}
Owner: base64 string - representing public key of user
```

###### Body
JSON as base64 string
```javascript
Data held by the Structured Data as a base64 string
```


#### Update

Structured Data can be updated by passing the `Id, tagType and SD-Version` corresponding
to the Structured Data.

For example, Say two users using an application request Structured Data with the ID ABC,
type tag 9. Assuming both the users get the same `SD-Version as 5`, which means both have the same copy of the Structured Data. One user updates the Structured Data a few times and the `SD-Version`
is now at `8`.
When the other user who still has `SD-Version 5` - when he tries to update -the API must be able to throw a proper status code describing the conflict in SD-Version (409). Based on which the applications can get the latest Structured Data and update the same again. If the `SD-Version` is
not specified in the request, the latest Structured Data will be updated with the data passed in the update, which may lead to a loss of modifications which might have happened in the mean time.

##### Request

###### End point
```
/structuredData/{id}/{tagType}/{SD-Version}
```
|Field| Description|
|-----|------------|
|id   | u8 array of length 64 as a base64 string.|
|tagType| Must be a permitted u64 value (9 - 11 or 10001 - (2^64 - 1)).|
|SD-Version| Optional value - Checks whether the latest version of Structured Data is being updated, if the value is set to true, otherwise it will overwrite with the latest data.|


###### Method
```
PUT
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

###### Body
```
Data to be saved in the Structured Data as base64 String
```

##### Response

###### Header
```
Status: 200 Ok
```

###### Body
Only for Versioned Structure Data
```
Version Id as base64 string
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

###### Headers
```
Authorization: Bearer <TOKEN>
```

##### Response

###### Header
```
Status: 200 Ok
```

### Raw Data

Raw data can be saved in the network as Immutable Data through the self-encryption process.
When raw data is written to the network, the data is self-encrypted and split into smaller
chunks and saved as Immutable Data. The self-encryption process returns a DataMap, using which
the actual data can be retrieved.

The DataMap obtained is saved to network as Immutable Data and the ID of the Immutable Data is used to
refer to the DataMap. This will make it easier and avoid passing the serialised DataMap to and fro
between the launcher and the application.

After a create or update operation a new ID relating to the DataMap will be returned.

#### Create

This DataMap is saved in the Network as an Immutable Data and the ID of the Immutable Data is returned.
The DataMap can be encrypted using the user's key and stored, else it can stored without encryption
making it readable for public.

##### Request

##### Endpoint
```
/rawData/{isEncrypted}
```

|Field|Description|
|-----|-----------|
|isEncrypted| The DataMap will be encrypted and saved in the network, else it would be saved without encryption. Defaults to false|

##### Method
```
POST
```

###### Headers
```
Authorization: Bearer <TOKEN>
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

#### Get Meta Data

##### Request

##### Endpoint
```
/rawData/{id}/{isEncrypted}
```

|Field|Description|
|-----|-----------|
|isEncrypted| true or false based on how the Raw Data was initially created. Defaults to false|
|id| ID referring to the DataMap, obtained after the create operation.|
|offset| Optional parameter - if offset is not specified the data is appended to the end of the DataMap.|

##### Method
```
HEAD
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

#### Response

##### Header
```
status: 200 Ok
```

##### Body
```javascript
{
  length: u64 // Actual length of the raw data
}
```

#### Update

##### Request

###### Endpoint
```
/rawData/{id}/{isEncrypted}?offset=0
```

|Field|Description|
|-----|-----------|
|isEncrypted| true or false based on how the RawData was initially created. Defaults to false|
|id| ID referring to the DataMap, obtained after the create operation.|
|offset| Optional parameter - if offset is not specified the data is appended to the end of the DataMap.|

###### Method
```
PUT
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

#### Response

##### Header
```
status: 200 Ok
```

###### Body
```javascript
ID [u8;64] as bas64 string
```

#### Get

##### Request

###### Endpoint
```
/rawData/{id}/{isEncrypted}?offset=0&length=100
```

|Field|Description|
|-----|-----------|
|isEncrypted| true or false based on how the Raw Data was initially created. Defaults to false|
|id| ID obtained after the create/update operation.|
|offset| Optional parameter - if offset is specified, the data is read from the specified position. Else it will be read from the start|
|length| Optional parameter - if length is not specified, the value defaults to the full length.|

###### Method
```
GET
```

###### Headers
```
Authorization: Bearer <TOKEN>
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

### Utility APIs

#### Hybrid Encryption

Combined Asymmectric and Symmetric encryption. The data is encrypted using random Key and
IV with Xsalsa-symmetric encryption. Random IV ensures that same plain text produces different
cipher-texts for each fresh symmetric encryption even with the same key. The Key and IV are then asymmetrically
enrypted using Public-MAID and the whole thing is then serialised into a single Vec<u8>.

##### Request

###### End Point
```
/util/encrypt
```

###### Method
```
POST
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

###### Body
```
Byte Array to be encrypted as base64 String
```

##### Response

###### Headers
```
Status: 200 Ok
```

###### Body
```
Encrypted byte array as base64 String
```

#### Hybrid Decryption

Decrypt data that was encrypted using the hybrid encryption API.

###### End Point
```
/util/decrypt
```

###### Method
```
POST
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

###### Body
```
Encrypted byte array as base64 String
```

##### Response

###### Headers
```
Status: 200 Ok
```

###### Body
```
Decrypted byte array as base64 String
```

#### Get Public Key

Public key will refer to the owner of the Structured Data. The public key can be
used to assert the ownership of Structured Data.

##### Request

###### End Point
```
/util/publicKey
```

###### Method
```
GET
```

###### Headers
```
Authorization: Bearer <TOKEN>
```

##### Response

###### Headers
```
Status: 200 Ok
```

###### Body
```
sodiumoxide box public key as base64 String
```

# Drawbacks

1. Large file sizes cannot be supported. Streaming API will be needed for
supporting large file content.
2. Raw data cannot be completely re-written. It can only be updated (partial update)
or appended. A workaround will be to create a new DataMap.
3. Multi Signature support is not exposed in the API.
4. Version of the Structured Data can not be deleted.

# Alternatives

None

# Unresolved Questions

When Structured Data is saved in the network directly using the low level API,
the Launcher will not be able keep an account of the Structured Data saved by the user.
Here the application will only be able to fetch the data from the network and the
users will be able to manage the data only through the application and not via the Launcher.
Should the Launcher keep track of the Structure Data and Immutable Data created by the
user?
