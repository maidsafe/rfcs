# Launcher Data Types API

- Status: rejected
- Type: new feature
- Related components: safe_launcher
- Start Date: 20-11-2015
- RFC PR: #75
- Issue number: Proposed - #77

# Summary

Expose API from launcher to perform operations on Structured data and
Immutable data through the launcher API.

# Motivation

Structured data is designed in such a way to enable the developers to create their own
data structures.

For example, safe_nfs uses Structured data for storing the DirectoryListing in the SafeNetwork.
safe_dns uses the logic as detailed here in this [RFC](https://github.com/maidsafe/rfcs/blob/master/implemented/0002-name-service/0002-name-service.md#detailed-design) to save the name service in the SafeNetwork.

Providing these APIs would open up the possibilities of using the structured
data to fit the needs of the developers to manage their own data structures,
which would facilitate in building wide range of applications on SafeNetwork.

Structured data is detailed in this implemented [unified structured data RFC](https://github.com/maidsafe/rfcs/blob/master/implemented/0000-Unified-structured-data/0000-Unified-structured-data.md#structured-data).

# Detailed design

The APIs that would facilitate operations on the SafeNetwork Data types can be grouped
under the end point `safe-api/v1.0/data_types/`.

## Structured data API

#### Create Structured data
Structure Data with a custom `type_tag` can be created only within the permitted range as specified in the [reserved-names RFC](https://github.com/maidsafe/rfcs/blob/master/implemented/0003-reserved_names/0003-reserved_names.md#detailed-design),
i.e within the permissible range 10,001 to 2^64

##### Request
```javascript
{
  endpoint: 'safe-api/v1.0/data_types/create_structured_data',
  data: {
    id: String, // Id of the structured Data
    type_tag: Integer,
    content: String, // data to be saved
    is_private: Boolean // If private the data would be encrypted
  }
}
```
##### Response
 Returns error with code and description. If the request is processed successfully
 then the error is returned as null

###### On Success
```javascript
{
  id: String, // base64 String
  error: null
}
```

###### On Error
```javascript
{
  id: String, // base64 String
  error: {
    code: -200,
    description: 'Some Error'
  }
}
```

#### Get structured data
##### Request
```javascript
{
  endpoint: 'safe-api/v1.0/data_types/get_structured_data',
  data: {
    id: String, // base64 String
    type_tag: Integer,
    is_private: Boolean
  }
}
```

##### Response

###### On Success
```javascript
{
  id: String, // base64 String
  data: {
    version: Integer, // version number of the Structured Data
    owners: [ String ], // List of base64 String representing the public signing key of the Owners
    content: String // base64 String
  }
}
```

###### On Error
```javascript
{
  id: String,// base64 String
  error: {
    code: -200,
    description: 'Some Error'
  }
}
```

#### Update structured data
```javascript
{
  endpoint: 'safe-api/v1.0/data_types/update_structured_data',
  data: {
    id: String, // base64 String
    type_tag: Integer,
    version: Integer, // Incremented version number    
    content: String, // base64 String - New Content associated to the Structured Data
    owners: [ String ], // List of base64 String representing the public signing key of the Owners
    previous_owners: [ String ], // Optional field - List of base64 String representing the public signing key of the previous owners.
                                 // This is used while transferring the ownership of the Structured data
    is_private: Boolean // If private the data would be encrypted and saved in the network
  }
}
```

##### Response
 Returns error with code and description. If the request is processed successfully
 then the error is returned as null


###### On Success
```javascript
{
 id: String, // base64 String
 error: null
}
```

###### On Error
```javascript
{
 id: String, // base64 String
 error: {
   code: -200,
   description: 'Some Error'
 }
}
```

## Immutable data

Raw Data in the network is stored as Immutable data after the SelfEncryption process.

### Save Raw data

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/data_types/save_raw_data',
  data: {
    content: String // base64 String
  }
}
```

##### Response
 Returns error with code and description. If the request is processed successfully
 then the serialised [DataMap](https://github.com/maidsafe/self_encryption/blob/master/src/datamap.rs#L44) is returned as base64 String

###### On Success
```javascript
{
 id: String,
 data: String // base64 String - serialised DataMap
}
```

###### On Error
```javascript
{
 id: String,
 error: {
   code: -200,
   description: 'Some Error'
 }
}
```

### Update raw data

Update an already existing raw data in the network by pasing the DataMap corresponding to it.

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/data_types/update_raw_data',
  data: {
    datamap: String, // base64 String - Serialised DataMap
    content: String // base64 String
    offset: Integer // Optional field - Offset from where the data is to be written. If not specified then the data is appended to the last
  }
}
```

##### Response
 Returns error with code and description. If the request is processed successfully
 then the Updated serialised [DataMap](https://github.com/maidsafe/self_encryption/blob/master/src/datamap.rs#L44) is returned as base64 String

###### On Success
```javascript
{
 id: String,
 data: String // base64 String - serialised DataMap
}
```

###### On Error
```javascript
{
 id: String,
 error: {
   code: -200,
   description: 'Some Error'
 }
}
```

### Get raw data

#### Request
```javascript
{
  endpoint: 'safe-api/v1.0/data_types/get_raw_data',
  data: {
    content: String // base64 String - Serialised DataMap
  }
}
```

##### Response
Returns error with code and description. If the request is processed successfully
then the data associated with the DataMap is returned

###### On Success
```javascript
{
 id: String,
 data: String // base64 String
}
```

###### On Error
```javascript
{
 id: String,
 error: {
   code: -200,
   description: 'Some Error'
 }
}
```

# Drawbacks
If a big file/data has to be saved from the application (Say 5gb). Then
writing the entire content of the data through the API at one go would choke up the resources.
As a workaround the `update raw data` API can be invoked in smaller chunks, but this approach will increase the number of `GET` calls to the network.

# Alternatives
To overcome the drawback, we can probably derive motivation from the REST based cloud APIs such as AWS.
The way they handle large file uploads is bit different.

1. API is invoked to indicate a beginning of an upload along with the actual content size that would be uploaded. This will return an temporary token or a request id.
2. Then the actual upload of the content starts in smaller chunks, this request will refer to the temporary token obtained from step1.
3. The upload is continued till the file is completely uploaded.

A similar approach can be adopted as an enhancement later, without disturbing the existing API.   

# Unresolved questions

None
