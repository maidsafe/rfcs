# Cipher-Opts

## Get Cipher Opts Handle
Unauthorised access is permitted for fetching PLAIN handle type

### Request

#### Endpoint
|Field|Description|
|-----|-----------|
|type| Enum. Values - PLAIN, SYMMETRIC, ASYMMETRIC|
|encryptKeyHandleId| Mandatory only for ASYMMETRIC type|

```
GET cipher-opts/{type}/{encryptKeyHandleId}
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

## Drop handle

### Request

#### Endpoint
```
DELETE cipher-opts/{handleId}
```

### Response

#### Status code
```
200
```
