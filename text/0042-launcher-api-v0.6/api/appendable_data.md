# Appendable Data API

Appendable Data is explained in detail in the [RFC](https://github.com/maidsafe/rfcs/blob/master/text/0038-appendable-data/0038-appendable-data.md).  

Operation on the Appendable Data would require the handle for the Appendable Data.
Since a handle is passed to the application, the applications must drop the handle after its usage.

## Create
Appendable data can be either public or private type.

|Type|Description|
|----|-----------|
| Public | data appended can be read by others |
| Private | data appended to it can only be read by the owner |

The appendable data can be fetched from network based on its Id. The Id is a base64 string
representing [u8; 32] array.

The appendable data also provides option to filter users by public key. Two filter types are supported,
Whitelist(allow) or Blacklist(restrict).

After creating the Appendable data the PUT endpoint must be invoked for saving the appendable data in the network

### Request

#### Endpoint

```
POST /appendable-data/
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
|Field|Description|
|-----|-----------|
| name | base64 string representing [u8; 32] array |
| isPrivate | Boolean. Optional Defaults to false |
| filterType | Enum value. WHITE_LIST, BLACK_LIST. Defaults to BLACK_LIST |
| filterKey | List of signing key handles. Optional value defaults to empty list |

```
{
  name: base64 string,
  isPrivate: Boolean,
  filterType: Enum,
  filterKeys: list of signing key handles
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
  handleId:u64 number representing the appendable data handle id
}
```

## Save AppendableData

### Request

#### Endpoint

##### PUT Endpoint
```
PUT /appendable-data/{handleId}
```

##### POST Endpoint
```
POST /appendable-data/{handleId}
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

## Get AppendableData Handle from DataIdentifier Handle

Get the handle for appendable data. Unauthorised access is allowed.

### Request

#### Endpoint
```
GET /appendable-data/handle/{DataIdentifier-Handle}
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
  handleId: u64 number representing the appendable data handle id,
  isOwner: Boolean,
  version: u64,
  filterType: ENUM,
  dataLength: Number,
  deletedDataLength: Number
}
```


## Get metadata

Get metadata of appendable data. Unauthorised access is allowed.

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
  version: u64,
  filterType: ENUM,
  dataLength: Number,
  deletedDataLength: Number
}
```

## Get encryption key

Get encryption key of the owner for private appendable data.
Handle for the encryption key is returned.

### Request

#### Endpoint

```
GET /appendable-data/encrypt-key/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Body

```
{
  handleId: Number // handle id as u64
}
```

## Get Signing key of a data by index

The handle for the signing key is returned

### Request

#### Endpoint

```
GET /appendable-data/sign-Key/{Handle-Id}/{index}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Body

```
{
  handleId: u64 // representing Signing key handle
}
```

## Get Signing key of a deleted data by index

Get Signing key from deleted section of the appendable data by index

### Request

#### Endpoint

```
GET /appendable-data/sign-key/deleted-data/{Handle-Id}/{index}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Body
```
{
  handleId: u64 // representing signing key handle
}
```

## Drop Sign key handle

Drop Sign key handle. Unauthorised access is allowed

### Request

#### Endpoint

```
DELETE /appendable-data/sign-key/{Handle-Id}
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

## Get DataIdentifier Handle

### Request

#### Endpoint

```
GET /appendable-data/data-id/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Body
```
{
  handleId: u64 // representing DataIdentifier handle
}
```

## Get Data Id of a Data at Appendable Data

Returns the dataid handle of a data based on the index from the Appendable data's data section.

Unauthorised access is allowed for Public AppendableData

### Request

#### Endpoint

```
GET /appendable-data/{Handle-Id}/{index}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Body
```
{
  handleId: u64 // representing DataIdentifier handle
}
```

## Get Data Id of a Data at Appendable Data - deleted_data

Returns the dataid handle of a data based on the index from the Appendable data's data section.

### Request

#### Endpoint

```
GET /appendable-data/deleted-data/{Handle-Id}/{index}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

### Response

#### Body
```
{
  handleId: u64 // representing DataIdentifier handle
}
```

## Append Data

This will add to the appendable data and update to the network. No need to invoke POST endpoint
for this change to reflect.

### Request

#### Endpoint
```
PUT /appendable-data/{Handle-Id}/{DataIdentifier-Handle-Id}
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

### Toggle filter

The api will toggle the filter and also reset all the filter keys.

#### Endpoint
```
PUT /appendable-data/toggle-filter/{Handle-Id}
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

### Add sign keys to filter

#### Endpoint
```
PUT /appendable-data/filter/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
```
[] // list of signing key handles
```

### Response

#### Status Code
```
200
```

### DELETE sign keys from filter

#### Endpoint
```
DELETE /appendable-data/filter/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
```
[] // list of sign key handles
```

### Response

#### Status Code
```
200
```

### Remove from data by index

Remove data held in the appendable data based on index. This operation will move the data
to the `deleted` section in the AppendableData.

### Request

#### Endpoint
|Field|Description|
|-----|-----------|
|index| Index of the data to be deleted from appendable data. Defaults to 0|

```
DELETE /appendable-data/{Handle-Id}
```

### Headers
```
Authorization: Bearer <TOKEN>
```

### Body
```
[] // list of index to be removed
```

### Response

#### Status code
```
200
```

### Remove from deleted data by index

Remove deleted data held in the appendable data based on index.

### Request

#### Endpoint
|Field|Description|
|-----|-----------|
|index| Index of the data to be deleted from appendable data. Defaults to 0|

```
DELETE /appendable-data/deletedData/{Handle-Id}
```

### Headers
```
Authorization: Bearer <TOKEN>
```

### Body
```
[] // list of index to be removed
```

### Response

#### Status code
```
200
```

### Move all data to deleted section

### Request

#### Endpoint

```
DELETE /appendable-data/clear-data/{handleId}
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

### Clear deleted data section

### Request

#### Endpoint

```
DELETE /appendable-data/clear-deleted-data/{handleId}
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

## Restore from deleted data

### Request

#### Endpoint
```
PUT /appendable-data/restore/{Handle-Id}/{deletedIndex}
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

## Serialise AppendableData

### Request

#### Endpoint
```
GET /appendable-data/serialise/{Handle-Id}
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
POST /appendable-data/deserialise
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
  handleId: u64 number representing the appendable data handle id,
  isOwner: Boolean,
  version: u64,
  filterType: ENUM,
  dataLength: Number,
  deletedDataLength: Number
}
```

## Drop Handle
Unauthorised access is allowed.

### Request

#### Endpoint
```
DELETE /appendable-data/handle/{Handle-Id}
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
