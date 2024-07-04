# DataIdentifier

## Get DataIdentifier for StructuredData

Unauthorised access is permitted which enables to read plain data

### Request

#### Endpoint

```
POST data-id/structured-data
```

#### Body
```
{
  name: base64 String [u8;32],
  typeTag: 500, 501 or above 15000. Defaults to 500
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
  handleId: u64 representing the handle id
}
```

## Get DataIdentifier for AppendableData

Unauthorised access is permitted which enables to read public data

### Request

#### Endpoint

```
POST data-id/appendable-data
```

#### Body
```
{
  name: base64 String [u8;32],
  isPrivate: Boolean. Defaults to false
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
  handleId: u64 representing the handle id
}
```

## Serialise

### Request

#### Endpoint
```
GET data-id/serialise/{Handle-Id}
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
Serialised DataMap [u8]
```

## Deserialise
Unauthorised access is allowed.

### Request

#### Endpoint
```
POST data-id/deserialise
```

#### Headers
```
Authorization: Bearer <TOKEN>
```

#### Body
```
Serialised Data as [u8]
```

### Response

#### Status Code
```
200
```

#### Body
```
{
  handleId:u64 representing DataMap handle
}
```


## Drop handle

### Request

#### Endpoint
```
DELETE data-id/{handleId}
```

### Response

#### Status code
```
200
```
