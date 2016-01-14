- Feature Name: Launcher as a local server
- Type New Product
- Related components safe_Launcher
- Start Date: 08-01-2016
- RFC PR:
- Issue number:

# Summary

An alternate approach for the Launcher on desktops, to reduce the application configurations and also to facilitate easier
development using RESTFul APIs.

# Motivation

Launcher is responsible for starting the SAFE Network compatible applications. Launcher acts as a server gateway between
the application and the SAFE Network. The applications, authorise and exchange data with the SAFE Network using the IPC
calls exposed by the Launcher. This approach, certainly is more secure as it opens one secure channel for each
application to communicate. But at the same time the configuration required to kick start is not easy as one would
expect. Also the developers have to learn / understand how to make the IPC calls.

The end user group would benefit with reduced configuration / setup to launch applications. Moreover, exposing RESTFul APIs
would reduce the learning curve for the developers. Since REST APIs are served through the standard `HTTP` protocol, the
developers get to use the same set of tools for development and debugging without any difference.

Since HTTP is a common and widely used application protocol, most of the platforms offer out of the box support.


# Detailed design

The user would register with the network and login using the registered network credentials. Once the user is
authorised, a server would be started locally at a fix port for serving the RESTFul APIs for the applications to connect.
The Launcher is now ready for applications to connect, no configuration is needed (good to go!).
When the user starts a SAFE Network compatible application, the application would request the Launcher for authorisation
to connect along with the set of permissions that the application would need, for example, an application can request for
authorisation along with the permission for SAFE Drive Access. When the request for authorisation is received by the
Launcher, the Launcher would present the request to the user for manual approval (though a prompt / model dialog). The user
at this point decides whether to approve or reject the authorisation. If approved the Launcher would pass a token to the
application. The applications can invoke the RESTFul APIs to perform operations on the network by passing the token in
the request's `Authorization` header field.

The applications get authorised after the manual approval and a token based authorisation for API calls.

### RESTFul API

APIs for authorisation and network operations would be exposed by the local server through a RESTFul interface.

#### End point design

The endpoint structure in general for the API would be `{version}/{module}/{path}`

`version` would refer to the API version that is being exposed. This would help in providing backward compatibility
support. When the APIs are upgraded, the Launcher would support the latest and also the last stable version
of the APIs. Only one backward version would be supported by the Launcher. This approach would be better suited to
slowly deprecate the old API versions and at the same time give the needed time for the applications to upgrade itself
to the latest version.

`module` would refer to the specific module to which the request is targeted. For example: auth, nfs, dns could be a few
modules.

`path` would refer to action that would be performed on the module. `path` is an optional part in the end point.

For example, to get a file through the dns module a GET request like
`HTTP GET /v1/dns?domain=maidsafe&service=www&path=index.html` could be sufficient and there is no path defined in this
request. But at the same time for fetching the list of the registered dns records for the user, the API can be
`HTTP GET /v1/dns/list`. Since there are two GET requests mapped to a module the `path` helps in differentiating the GET
requests to perform the specific action for the request.
The APIs are just taken as an example, detailed module specific API are explained in a separate document.

#### Authorisation API

The applications can request the Launcher for authorised access. But at the same time few modules can expose `GET` requests
without authorisation, such `GET` requests without any authorisation would use an unregistered client to get the data
from the SAFE Network. Unregistered client access might be needed by applications such as browsers to fetch the **public and
unencrypted** content from the network.

##### Authorised Access

When an application requests for authorisation with the Launcher, the user has to manually approve the authorisation
request. On approval the application would get a token specific for the application from the Launcher. The life of the token
is until the Launcher is running or till the user manually revokes the token.

The sequential steps in the authorisation process:

1. When the application is started, an asymmetric key pair and a nonce is generated for securely exchanging the symmetric
encryption key with the Launcher (ECDH-Key-Exchange). This uses Curve25519 (from libsodium) for symmetric key exchange.
2. Every application will have to provide its own unique identifier string. Consider a vendor `ABC` has applications
`Photo app` & `Video app`. Both the applications will have a unique string under the same vendor, i.e. `Photo app` and
`Video app` should not have the same id. This unique id provided can not be changed in future. If the id is changed then
the data saved in the application directory would be lost. In case both applications under the vendor share the same
application identifier then the both applications would share the same application directory. The purpose of this
unique key / id is explained in the [Application Directory Handling](#application-directory-handling) section.
3. Now the application will make a POST request to the authorisation end point. The list of permissions required for the
applications are also sent along with the authorisation request.

  ###### Request
  **POST** /v1/auth/authorise
  ```javascript
  {
   application: {
     name: 'DNS Example', // Required
     vendor: 'MaidSafe', // Required
     id: string, // Required - unique string to identify the application - this can not be changed later - if changed
                 // it will lead to loss of data
     version: 0.0.1 // Required
   },
   permissions: [ // Optional
    'SAFE_DRIVE_ACCESS'
   ],
   publicKey: base64String // Public Key for Asymmetric encryption
   nonce: base64String // Asymmetric Nonce
  }
  ```
4. Once the authorisation request is received by the Launcher, the request is processed and prompted for a user's approval
in a human readable format.

  ```
       ---------------------------------------------------
       |                                                 |
       | DNS Example is requesting access.               |
       |                                                 |
       | Permissions Requested:                          |
       |  SAFE DRIVE ACCESS                              |
       |                                                 |
       | Vendor: MaidSafe                                |
       | Version: 0.0.1                                  |
       |                                                 |
       |        Yes                     No               |
       |                                                 |
       ---------------------------------------------------  
  ```
5. If the user rejects the request, then the authorisation failure response is sent back to the application

  ###### Response
  HTTP Status code: 401 (Unauthorised)
6. If the user approves the request, then a random unique session id is generated.
7. Symmetric key and nonce is generated using libsodium. This key is used for encrypting the messages between the application
and the Launcher through the REST API.
8. JSON web token is generated using the session id (step 6) and the symmetric encryption key (step 7),
    ```
    header - { "type": "JWT", alg: "HS256"}
    payload - {"id": "sessionId"}
    ```
The token is signed using the secret symmetric key (from step 7).
9. The session related information is stored in the memory. The session id would serve as a key for fetching the session
information. The session information would contain the details received from authorisation request and the symmetric
key is also stored.
10. An asymmetric key pair is generated for the symmetric key exchange.
11. The symmetric key (from step 7) is encrypted using the application nonce, application public key and the private key
generated in step 10. The application nonce and application public key is received as a part of the authorisation request.
12. The response is constructed and sent back to the application.

  ###### Response

  ###### Header
  Status Code: 200
  Content-Type: application/json

  ###### Response Body
  ```javascript
  {
     token: generated_jwt_token,
     encryptedSymmetricKey: base64String, // [SYMMETRIC_KEY + NONCE]
     public_key: base64String,
     permissions: [
      'SAFE_DRIVE_ACCESS'
     ]
  }
  ```
13. Once the authorisation is completed, the APIs can be invoked by passing the JWT token in the request
header.

    ###### Sample request
    ```
    HTTP GET v1/dns?domain=maidsafe
    ```
    ###### Request headers
    ```
    Authorization: Bearer {JWT TOKEN after authorisation (from step 12)}
    ```
14. For an authorised request the query string and the payload (http body) should be encrypted using the symmetric Key.

##### Validating the Token

When an authorised request is received by the Launcher,

1. The JSON web token is extracted from the `Authorization` header field of the request.
2. The payload part of the token is converted to a JSON object and the session id is obtained.
3. The symmetric Key is retrieved from the session info saved in the memory.
4. The token is validated by verifying the signature using the symmetric key.
5. If the signature is valid then the next action in performed, or else a 401 status code is sent to the client.
6. The list of permissions granted for the application can also be fetched from the session info details

#### Application Directory Handling

The applications might need a directory to store its own data. Thus the network would create a directory for the
application when it is authorised for the first time by the user.

The user session packet will have a reference to the reserved `APPLICATION_METADATA` directory. Application related
metadata will be represented in CBOR format and saved in the `APPLICATION_METADATA` directory.

##### Sample CBOR Representation

```javascript
{
  APP_ID_1: {
    directory_key : ::safe_nfs::metadata::DirectoryKey
  },
  APP_ID_2: {
    directory_key : ::safe_nfs::metadata::DirectoryKey
  }
}
```

Application ID is generated using a deterministic approach. Every application will provide its own unique key / identifier
string along with the vendor name in the authorisation request. The hash of vendor name and unique key provided by the
application (SHA512(Vendor + AppKey)) will yield the id of the application.

When a user approves an application,

1. The `metadata.cbor` file is fetched from the `APPLICATION_METADATA` directory.
2. Application id is generated by hashing the vendor name and the application unique key from the authorisation request.
3. If the application id is not present in the current configuration, a new directory is created in the network.
This directory key is mapped with the generated application id and saved in the `metadata.cbor` file.
4. If the application id is already present then the directory for application can be fetched form the network using the
directory key


#### Proxy Component

Web application development support is very important. Most of the developers opt for web based applications
because the same code can work across platforms / devices. Having said that, the same source code has to work on different
platforms / devices, the addon / extension approach wouldn't scale easily.

Using a **local HTTP proxy** would make it simple for end users and the developers. Again, the emphasis is
made to use the standard `HTTP`, so that even on other platforms (mobile webkits) the HTTP request can be intercepted
and the content can be served as needed.  

The browsers can be configured to use the local proxy only for requests with the `.safenet` TLD. The configuration can
be simplified by providing a PAC file.

When a user types `maidsafe.safenet` in the browser's address bar, the request is received by the proxy. The proxy would
redirect the request to the RESTFul API to serve the content. The request `maidsafe.safenet` would be converted to
`\dns\file?service=www&domain=maidsafe&file=index.html` and redirected to the RESTFul server for serving the content.

The Web applications can also request for authorisation by invoking the auth API. The developers are expected
to use `api.safenet` as the host name for the REST endpoints instead of directory using `localhost:PORT`.
For example: `http:\\api.safenet\dns\file?service=www&domain=maidsafe&file=index.html` is preferred.

Following a fixed standard for host name will help in supporting the same code to work across platforms. For instance,
the Launcher on mobile can be a different implementation approach (probably not a RESTFul server) because of the
sand boxing restrictions. Fixing a standard host name for the API end points, will allow to intercept the requests
specifically and handle based on the platform to serve the content from the network.

### FFI interface

The server component can be built using any best fit tool. The server would make use of the ffi bindings (safe_ffi)
to perform network operations.


# Drawbacks

Though the development and initial configurations are made easier, the user experience might not be great because the
user has to manually click on the prompt to authorise each and every time. An option to persist the authorisation
can be a feature that can be added in future to improve the experience.

# Alternatives

None

# Unresolved questions

The deterministic approach of creating an application id depends on the unique application key and the name of the vendor.
This can be improved once we know how the network would reward the application / vendor.
