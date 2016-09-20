# Immutable Data API

Immutable data is written to and read from the network via self-encryption.

## Write Immutable Data using self-encryptor

Once the write operation is successful, the api returns a Handle-Id corresponding
to the DataMap. The Handle-Id refers to the pointer held in memory referring to
the DataMap, this Handle-Id can be used to work with the DataMap.

### Request

#### Endpoint

```
POST /self-encrypt
```

#### Headers

```
Authorization: Bearer <TOKEN>
Content-Length: The length of the request body
```

#### Body

```
Binary data
```

### Response

#### Status Code

```
200
```

#### Headers

```
Handle-Id: u64 representing DataMap handle
```

## Get actual size of data from Handle-Id
Unauthorised access is allowed.
### Request

#### Endpoint
```
HEAD /self-encrypt/{Handle-Id}
```

#### Header

```
Authorization: Bearer <TOKEN>
```

### Response

#### Status Code

```
200
```

#### Header
|Field|Description|
|-----|-----------|
|Content-Length| Size of the file in bytes|

```
Content-Length: Number
```

## Read using self-encryptor

API to read the binary data from the network by passing the DataMap-Handle ID.
Unauthorised access is allowed.

### Request

#### Endpoint

```
GET /self-encrypt/{Handle-Id}
```

#### Headers
```
Authorization: Bearer <TOKEN>
Range: bytes=0- // Optional range for partial read
```

### Response

#### Status Code
```
200 or 206
```

#### Header
```
Accept-Ranges: bytes
Content-Length: <Length that is requested based on the byte range>
Content-Range: bytes <START>-<END>/<TOTAL>
```

#### Body
```
Binary data
```
