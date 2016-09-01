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
| Unversioned | 500 | Has only the one latest copy |
| Versioned | 501 | Will hold version history. Can fetch an older version based on a version number|
| Custom | 15000 > | Apps are free to use any tagType value greater than 15000 |


The data is be encrypted based on the Encryption enum value specified.

|Encryption| Description|
|-----|-----------|
|NONE| Data is not encrypted|
|SYMMETRIC| Data is encrypted using symmetric key - only the user can read the data|
|HYBRID| Data is encrypted using hybrid encryption for sharing the data secure|

The hybrid encryption is detailed in the [safe_core low level api RFC](https://github.com/maidsafe/rfcs/blob/master/text/0041-low-level-api/0041-low-level-api.md)

The tag type and Id combination is needed for fetching the Structured Data from the network.

The Id is a base64 string representing [u8; 32] array.

The DataIdentifier handle is obtained after creation of a SD. The Data Identifier handle is
needed for working with the structured data and this must be dropped after the usage.

The [appendable data](./appendable_data.md) can accept only Data Identifier to appened.

### Request

#### Endpoint

```
POST /structured-data/{Id}
```

#### Headers
|Field|Description|
|-----|-----------|
|Tag-Type| Accepted values 500, 501 or above 15000. Defaults to 500 |
|Encryption| Enum values - NONE, SYMMETRIC, HYBRID. Defaults to None |

```
Authorization: Bearer <TOKEN>
Tag-Type: Number
Encryption: String
```

#### Body
```
Binary Data
```

### Response

#### Status Code

 ```
 200
 ```

#### Headers
```
Handle-Id: u64 representing the DataIdentifier handle Id
```

## Get Data Identifier Handle

Get the handle for structured data.
Unauthorised access is allowed.

### Request

#### Endpoint
```
HEAD /structured-data/handle/{Id}
```

#### Headers
|Field|Description|
|-----|-----------|
|Tag-Type| Accepted values 500, 501 or above 15000. Defaults to 500 |

```
Authorization: Bearer <TOKEN>
Tag-Type: Number
```

### Response

#### Status code

```
200
```

#### Headers

```
Is-Owner: Boolean
Handle-Id: base64 string representing the Id for the DataIdentifier handle
```

## Metadata

Get the metadata of structured data using handle id
Unauthorised access is allowed.

### Request

#### Endpoint
```
HEAD /structured-data/{Handle-Id}
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

#### Headers

```
Is-Owner: Boolean
```

## Get versions

Get the number of versions available for a versioned structured data (tag type 501).

### Request

#### Endpoint
```
HEAD /structured-data/versions/{Handle-Id}
```

#### Headers
|Field|Description|
|-----|-----------|
|Encryption| Enum value - NONE, SYMMETRIC, HYBRID. Optional value. Defaults to NONE |


```
Authorization: Bearer <TOKEN>
Encryption: Enum // optional
```

### Response

#### Status code

```
200
```

#### Headers

```
Versions-Length: Number
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
Encryption: Enum // optional
```

### Response

#### Status Code
```
200
```

#### Headers
```
Versions-Length: Number
Version-Number: Number // Version number of the SD currently being served
Is-Owner: Boolean
```

#### Body
```
Binary Data
```

## Update SD

### Request

#### Endpoint
```
PUT /structured-data/{Handle-Id}
```

#### Headers
|Field|Description|
|-----|-----------|
|Encryption| Enum values - NONE, SYMMETRIC, HYBRID. Defaults to NONE |

```
Authorization: Bearer <TOKEN>
Encryption: String
```

#### Body
```
binary data
```

### Response

#### Status code
```
200
```

### Delete SD

- Delete of SD will clear the Handle too

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
