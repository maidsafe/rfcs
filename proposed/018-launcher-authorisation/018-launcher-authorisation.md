- Feature Name: Authorisation from safe_launcher
- Type: Enhancement
- Related components: safe_launcher
- Start Date: 01-12-2015
- RFC PR:
- Issue number:

# Summary

Providing a simpler authorisation mechanism via safe_launcher, so that the safe_launcher
would be able to handle application authorisation with much lesser configuration.
This is an enhancement of the existing authentication process.

# Motivation

The present workflow of the launcher, makes it a must to pre-configure the application
with the launcher. The user must posses a certain level of technical competency to locate
the application binary (Portable apps, installed apps, etc).

A simpler approach to start the applications without much configuration would make it
easy for end users to adopt to the SAFENetwork ecosystem.

# Detailed design

### Outlining the approach

1. Once the user logs in, the launcher would start a fix TCP port (59999). Will call this
as the `AUTHORISATION_PORT`
2. Application when started will send the authorisation request to the launcher
through the AUTHORISATION_PORT.
3. Launcher would prompt the user to manually confirm whether to allow the application to connect.
4. When the user approves, the launcher would send the connection parameters to the application.
5. Application would initiate the handshake process and start its communication with the launcher.

The only difference with this approach is in the way the authorisation is instantiated.
Previously the launcher would instantiate the authorisation. Now it is vice versa and without
any pre-configuration. The handshake process would be the same as detailed in the launcher-as-a-service RFC.

### Authenticated Access Workflow

1. User logs in to safe_launcher
2. Launcher would start a TCP socket at port **59999**. If the port is unavailable the launcher
won't start itself. This port would be the initial standard port through which the applications
would request for authorisation.
3. Say the user wants to start the safe_dns example application. The user would open/start the
application. When the application starts, it would make an authorisation request to the launcher
at the AUTHORISATION_PORT. Timeout for the response should be set to maximum because the
launcher would respond only after the user input (Allow/Deny).
```javascript
{
  endpoint: 'safe-api/v1.0/handshake/authorise',
  data: {
    app: {
      name: 'SAFE_DNS', // Name of the Application
      vendor: 'MaidSafe' // Developer Name
    },
    permissions: [ // OPTIONAL field - list of permissions that the application would need
      'SAFE_DRIVE_ACCESS'
    ]
  }
}
```
4. Launcher will prompt the user to confirm whether to allow the application to connect
with the requested permissions. The prompt would display the application name that is
requesting for authorisation along with the list of permission requested for. The user
would be able to `Allow` or `Deny` the request.
5. The launcher would reply to the request, based on the user's decision. If the user
denies, then the error response is sent. Else the connection parameters are sent.
###### On permission granted
```javascript
{
  id: [],
  data: {
    type: 'TCP',
    port: 9000,
    nonce: 'String',
    version: 1.0 // Version of the launcher
  }
}
```
###### On permission denied
```javascript
{
  id: [],
  error: {
    code: -100,
    description: 'Error description'
  }
}
```
6. The launcher would store the permissions requested. Permissions granted would be used
to validate later when the functional APIs are invoked.
7. Once the parameters are received by the application, the handshake can be carried as
described in the launcher-as-a-service RFC (#010).

### Un-Registered Client Access Workflow

There are plenty of use cases for unregistered clients (those that don't have a valid
SAFE Account with `MaidManagers`) to access the SAFE Network. One such example is the browser
category. The browsers do not need to create an account to access the SAFE Network nor do they
require a registered client engine (one that performs operations on a valid account) because
all they care about is the fetching and display of data. This is in line with our philosophy that
anyone can fetch data from the SAFE Network - it will be of no use if it is encrypted and client
fetching it does not have the decryption keys, but that is another matter. Without Launcher,
each such application will have to interface with low level libraries like [safe_core](https://github.com/maidsafe/safe_core) and/or [safe_nfs](https://github.com/maidsafe/safe_nfs). Further every instance of an engine
from [safe_core](https://github.com/maidsafe/safe_core) will create a new routing object.
All this is unnecessary overhead. Launcher will funnel requests from all unregistered
applications through a single instance of an unregistered client engine obtained
from [safe_core](https://github.com/maidsafe/safe_core).

1. User logs in to safe_launcher
2. Launcher would start a TCP socket at port **59999**. If the port is unavailable the launcher
won't start itself. This port would be the initial standard port through which the applications
would request for authorisation
3. Say the user wants to start the safe_dns example application. The user would open/start
the application. When the application starts, it would make an authorisation request to the launcher
at the AUTHORISATION_PORT.
```javascript
{
  endpoint: 'safe-api/v1.0/handshake/unregistered-access',
  data: {
    app: {
      name: 'SAFE_DNS', // Name of the Application
      vendor: 'MaidSafe' // Developer Name
    }
  }
}
```
4. The launcher would create a unregistered client instance and open a random port for
communication with the applications. The launcher will respond to the request with the
connection parameters or with an error in case of any unexpected exception. This port
would be a dedicated port for all the applications that would request for un-registered access.

###### On permission granted
```javascript
{
  id: [],
  data: {
    type: 'TCP',
    port: 9999,// Random dedicated port for unregistered-access    
    version: 1.0 // Version of the launcher
  }
}
```
###### On permission denied
```javascript
{
  id: [],
  error: {
    code: -100,
    description: 'Error description'
  }
}
```
5. All the communication that happen through this port will not be encrypted.
6. When an other application requests for unregistered-access, new port is not opened.
Instead the same port which is dedicated for un-registered access is passed to the application.

In short the unregistered-access port will be created only once and all the applications
would use the same port for unregistered-access. **The data transferred between the launcher
and the application will not be encrypted**

### Session management console

The launcher would maintain the list of authorised application which are connected with the launcher.
A console can be presented to the user through which the user can view the list of applications that are
currently connected along with the permissions that they have been granted. Even the applications
that requested for unregistered-access can be grouped.

# Drawbacks

The hash of the binary can not be validated in this approach. Previously we used to start the application
from the launcher and validating the hash was planned to detect binary swapping. But in this approach,
the user is starting the application.


# Alternatives

None

# Unresolved questions

How will the applications be rewarded by the network? Based on this, the application identification
at the initial authorisation request could be improved.
