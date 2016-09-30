# Immutable Data API

Immutable data is written to and read from the network via self-encryption.

## Get ImmutableData Reader

Unauthorised access is allowed.

### Request

#### Endpoint

```
GET /immutable-data/reader/{DataIdentifier-Handle-Id}
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
{
  handleId: u64, // representing the ImmutableDataReader
  size: u64
}
```

## Read Immutable Data using Reader

Supports streaming and partial read using the range header. Unauthorised access is allowed.

### Request

#### Endpoint

```
GET /immutable-data/{Reader-Handle-Id}
```

#### Headers

```
Authorization: Bearer <TOKEN>
```

### Response

#### Status Code

```
200 or 206
```

#### Body

```
Binary data
```


## Close Immutable Data Reader

Unauthorised access is allowed.

### Request

#### Endpoint

```
DELETE /immutable-data/reader/{Reader-Handle-Id}
```

#### Headers

```
Authorization: Bearer <TOKEN>
```

### Response

#### Status Code

```
200 or 206
```


## Get ImmutableData Writer

### Request

#### Endpoint

```
GET /immutable-data/writer
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
{
  handleId: u64, // representing the ImmutableDataWriter  
}
```

## Write Immutable Data

Supports streaming. Close writer must be invoked to ensure the data is saved completel

### Request

#### Endpoint

```
POST /immutable-data/{Writer-Handle-Id}
```

#### Headers

```
Authorization: Bearer <TOKEN>
Content-Length: <Length of data to be written>
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

## Close Immutable Data Writer

Invoking close on writer will generate the DataMap and return the DataIdentifier handle.

### Request

#### Endpoint

```
DELETE /immutable-data/writer/{Writer-Handle-Id}/{cipher-opts-handle}
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
