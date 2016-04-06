- Feature Name: Exposing Low level APIs for Structured Data and Immutable Data handling
- Type New Feature
- Related components safe_launcher, safe_ffi, safe_core
- Start Date: 06-04-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Proposal on exposing low level APIs from launcher that will allow app devs to use Structured Data
and Immutable data to create their own topologies.

# Motivation

Access to structured data and raw data will be needed for third party applications, which
will allow them to create their own data structures and also to store raw data.

# Detailed design

NFS provides a confined structure for Directory and File handling, but this topology
might not be the practical data structures that application might need in real time. Hence,
exposing the low level apis which will allow app devs to use the Structured Data to create and
manage their own data structures to build the applications.

To access the low level apis, the applications must request for `LOW_LEVEL_ACCESS` permission
at the time of authorisation with the launcher.

### Structured Data

Structured Data can be used to reference data in the network using an ID and the `tag_type`.
The Id of the structured data is a u8 array of length 64 [u8;64] and the `tag_type` value
can be between the range 10,001 to 2^64.

Few specific type tags in the reserved range can also be permitted. For example,
Appendable Structured Data Type can be one `tag_type` that can be permitted since it might be
needed for the third party applications.

At this point, `tag_type between the range 10,001 to 2^64` will be permitted by the launcher.
If any specific tag type within the reserved range has to be exposed then it can also be
added later to the permitted range list for the `tag_type` in launcher API.

#### Create

Versioned or un-versioned Structured Data can be created. Versioned Structured Data
will store versions of the Structured Data. Any modification to the Structured Data will
result in a new version of the Structured Data. Using versioned Structured Data, it is
possible to retrieve the older versions. While, Un-Versioned Structured Data will have only
the latest version.

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
  "isVersioned": Boolean // optional  - defaults to false
}
```

##### Response

###### Header
```
Status: 200 Ok
```

#### List versions

Retrieve the list of versions for the Structured Data. This will work only for the
Versioned Structured Data, else 400 (Bad Request) will be thrown. On success,
available version id list will be returned

##### Request

###### End point
```
/structuredData/{id}/{tagType}
```
```
id - Structured Data Id
tagType - tagType of the Structured Data
```

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
Retrieve the list of versions for the Structured Data. This will work only for the
Versioned Structured Data, else 400 (Bad Request) will be thrown. The response
will have data and a ref as a JSON. `ref` corresponds to the [version field in the structured Data](https://github.com/maidsafe/rfcs/blob/master/implemented/0000-Unified-structured-data/0000-Unified-structured-data.md#structureddata)
To avoid confusion it would be better to call this field with a different name as the api already uses version.
The `ref` value must be passed while updating the structured data. This value is useful for concurrency handling.
The ref will be internally mapped to the version field in the structured data. If the version being updated is outdated
then an error will be thrown (409 HTTP Status Code).


##### Request

###### End point
```
/structuredData/{id}/{tagType}/{isVersioned}
```
```
id - Structured Data Id
tagType - tagType of the Structured Data
isVersioned - Whether the Structured Data is versioned or not. This is an optional parameter,
defaults to false.
```

###### Method
```
GET
```
##### Response

###### Header
```
Status: 200 Ok
```

###### Body
JSON as base64 string
```
{
    ref: 1,
    data: Data held by the Structured Data as a base64 string
}
```

#### Get By Version

##### Request

###### End point
```
/structuredData/{id}/{tagType}/{versionId}
```
```
id - Structured Data Id
tagType - tagType of the Structured Data
versionId - id of the version of the Structured Data that has to be fetched.
```

###### Method
```
GET
```
##### Response

###### Header
```
status: 200 Ok
```

###### Body
JSON as base64 string
```javascript
{
    ref: 1,
    data: Data held by the Structured Data as a base64 string
}
```


#### Update

Structured data can be updated by passing the `Id, tagType and the ref` corresponding
to the structured data.

##### Request

###### End point
```
/structuredData/{id}/{tagType}/{ref}/{isVersioned}
```
```
id - Structured Data Id
tagType - tagType of the Structured Data
ref - The value received from the GET request for a Structured Data.
isVersioned - Whether the Structured Data is versioned or not. This is an optional parameter,
defaults to false.
```

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
the actual data can be retrieved.

#### Create

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
Base64 encoded Serialised content of DataMap
```

#### Update

##### Request

###### Endpoint
```
/rawData
```

###### Method
```
PUT
```

###### Body
```javascript
{
  dataMap: base64 String representing the serialised content of DataMap,
  offset: integer, // optional field. If the field is not present, data will be appended to the end
  data: actual data as base64 string
}
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
/rawData?offset=10&length=100
```
```
offset - Optional field. Defaults to 0
length - Optional field. Defaults to the full length.
```

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
supporting large file content

# Alternatives

None

# Unresolved questions

1. In nfs - directory helper line 340 - we are blindly saving the DL as Immutable Data
without validating any size. Is this fine?
2. Do we have delete implemented???
