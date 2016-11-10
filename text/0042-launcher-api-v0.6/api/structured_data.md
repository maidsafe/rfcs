# Structured Data API

## Create

Structured data has a size restriction of 100KiB including its internals.
If the size is more than permitted size after serialisation, error is returned.

Versioned, Unversioned or Custom Structured data types can be created.
If the size of the data being passed is greater than 100KiB, the versioned and unversioned
structured data will ensure that the data is managed and stored successfully.
But for custom type_tags it becomes the responsibility of the application
to handle the size restriction.


|Type| tag| Description|
|-----|---------|-----------|
| Versioned | 500 | Will hold version history. Can fetch an older version based on a version number |
| Unversioned | 501 | Has only the one latest copy |
| Custom | 15000 > | Apps are free to use any typeTag value greater than 15000 |


The data is be encrypted based on the Encryption enum value specified.

|Encryption| Description|
|-----|-----------|
|NONE| Data is not encrypted|
|SYMMETRIC| Data is encrypted using symmetric key - only the user can read the data|
|ASYMMETRIC| Data is encrypted using asymmetric encryption for sharing the data secure|

The asymmetric encryption is detailed in the [safe_core low level api RFC](https://github.com/maidsafe/rfcs/blob/master/text/0041-low-level-api/0041-low-level-api.md)

The type_tag and Id combination is needed for fetching the Structured Data from the network.

The Id is a base64 string representing [u8; 32] array.

The StructuredData handle is needed for working with the structured data and this
must be dropped after the usage.

### Request

#### Endpoint

```
POST /structured-data
```

#### Headers

```
Authorization: Bearer <TOKEN>
```

#### Body
|Field|Description|
|-----|-----------|
|name| [u8;32] as Base64 string |
|typeTag| Accepted values 500, 501 or above 15000. Defaults to 501 |
|encryption| Enum values - NONE, SYMMETRIC, ASYMMETRIC. Defaults to None |
|encryptKey| Encryption Key handle to use for asymmetric encryption  |

```
{
  name: [u8; 32] of base64 string,
  typeTag: Number, // options defaults to 501
  cipherOpts: u64 representing cipher-opts handle // optional defaults to PLAIN
}
```

### Response

#### Status Code

```
200
```

#### Body

```
{
  handleId: u64 representing the StructuredData handle Id
}
```

## Get Structured data Handle

Get the handle id for structured data using DataIdentifier handle.
Unauthorised access is allowed.

### Request

#### Endpoint
```
GET /structured-data/handle/{DataIdentifier-Handle}
```

### Response

#### Status code

```
200
```

#### Body

```
{
  isOwner: Boolean,
  handleId: u64,// representing StructuredData handle
  version: u64, // version of the structured data  
  dataVersionsLength: Number // number of data versions - only for type_tag 501
}
```

## Get metadata

Get metadata of structured data. Unauthorised access is allowed.

### Request

#### Endpoint
```
GET /appendable-data/metadata/Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Status code

```
200
```

#### Body

```
{  
  isOwner: Boolean,  
  version: u64, // version of the structured data  
  dataVersionsLength: Number // number of data versions - only for type_tag 501
}
```

## Get DataIdentifier handle for Structured data
Unauthorised access is allowed.

### Request

#### Endpoint
```
GET /structured-data/data-id/{handleId}
```

### Response

#### Status code

```
200
```

#### Body

```
{  
  handleId: u864 representing DataIdentifier handle  
}
```

## Read Data

Reads data from Structured Data. If it is a versioned Structured Data, the latest version is read by default
unauthorised access is allowed.

### Request

#### Endpoint
|Field|Description|
|-----|-----------|
|Version-Number| Number of a specific version to be read. Optional Value. Passed only for versioned structured data. Defaults to latest version|

```
GET /structured-data/{Handle-Id}/{Version-Number}
```

#### Headers
|Field|Description|
|-----|-----------|
|Encryption| Enum value - NONE, SYMMETRIC, HYBRID. Optional value. Defaults to NONE |


```
Authorization: Bearer <TOKEN>
```

### Response

#### Status Code
```
200
```

#### Headers
```
Version: U64 // Version number of structured data
Data-Versions-Length: Number
Data-Version-Number: Number // Version number currently being served
Is-Owner: Boolean
```

#### Body
```
Binary Data
```

## Save Structured Data

The safe_core calls PUT to store the data in the network and POST to update
an existing data in the network. When a structured data is created for the first time
the application should call the PUT endpoint and while updating a StructuredData the
application should call the POST endpoint.

### Request

#### PUT Endpoint
```
PUT /structured-data/{Handle-Id}
```

#### POST Endpoint
```
POST /structured-data/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Status code
```
200
```

## Update data

Update the data held by the StructuredData

### Request

#### Endpoint
```
PATCH /structured-data/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
|Field|Description|
|-----|-----------|
|Encryption| Enum values - NONE, SYMMETRIC, HYBRID. Defaults to NONE |
```
{
  cipherOpts: u64 representing cipher-opts handle,  
  data: base64 String representing [u8]
}
```

### Response

#### Status code
```
200
```

### Delete Structured Data

Deletion of Structured Data will clear the Handle from memory

### Request

#### Endpoint
```
DELETE /structured-data/{Handle-Id}
```

### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Status code
```
200
```

## Drop Handle

Unauthorised access is allowed

### Request

#### Endpoint
```
DELETE /structured-data/handle/{Handle-Id}
```

### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Status code
```
200
```

## Serialise StructuredData

### Request

#### Endpoint
```
GET /structured-data/serialise/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Status Code
```
200
```

#### Body
```
Binary data [u8]
```

## Deserialise
Unauthorised access is permitted

### Request

#### Endpoint
```
POST /structured-data/deserialise
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
```
Binary data [u8]
```

### Response

#### Status Code
```
200
```

#### Body
```
{
  isOwner: Boolean,
  Version: u64,
  handleId: base64 string representing the Id for the StructuredData handle,
  dataVersionsLength: Number
}
```
