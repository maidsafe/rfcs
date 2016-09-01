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

### Request

#### Endpoint

|Field|Description|
|-----|-----------|
| Id | base64 string representing [u8; 32] array |

```
POST /appendable-data/{Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
|Field|Description|
|-----|-----------|
| isPrivate | Boolean. Optional Defaults to false |
| filterType | Enum value. WHITE_LIST, BLACK_LIST |
| filterKey | List of public keys. Public key as base64 string. Optional value defaults to empty list |

```
{
  isPrivate: Boolean,
  filterType: Enum,
  filterKeys: list of publickeys
}
```

### Response

#### Status Code
 ```
 200
 ```

#### Headers
```
Handle-Id: u64 number representing the appendable data handle id
```

## Get Data Identifier Handle

Get the handle for appendable data. Unauthorised access is allowed.

### Request

#### Endpoint
```
GET /appendable-data/handle/{Id}
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
Is-Private: Boolean
Handle-Id: u64 number representing the appendable data handle id
```

## Metadata

Get the metadata of the appendable data including the size of data held.
Unauthorised access is allowed.

### Request

#### Endpoint
```
HEAD /appendable-data/{Handle-Id}
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
Is-Private: Boolean
Data-Length: Number
```

## Read Appendable Data

Read data from appendable data at a specific index.
Unauthorised access is allowed.

### Request

#### Endpoint
|Field|Description|
|-----|-----------|
|index| Index of the data to be read from appendable data. Defaults to 0|

```
GET /appendable-data/{Handle-Id}/{index}
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

#### Headers
```
Index: Number // index number from the Appendable data list that is being read
Appended-By: public key of the appender as base64 String
```

#### Body
```
Binary Data
```

## Append Data

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

### Add public key to filter

#### Endpoint
|Field|Description|
|-----|-----------|
|Public-Key| Public key of the user to append to filter list as base64 string  |
```
PUT /appendable-data/filter/{Handle-Id}/{Public-Key}
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

### DELETE public key from filter

#### Endpoint
|Field|Description|
|-----|-----------|
|Public-Key| Public key of the user to be removed from filter list as base64 string  |
```
DELETE /appendable-data/filter/{Handle-Id}/{Public-Key}
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


### Delete data by index

Delete data held in the appendable data based on index

### Request

#### Endpoint
|Field|Description|
|-----|-----------|
|index| Index of the data to be deleted from appendable data. Defaults to 0|

```
DELETE /appendable-data/{Handle-Id}/{index}
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


### Delete appendable data

### Request

#### Endpoint
```
DELETE /appendable-data/{Handle-Id}
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
