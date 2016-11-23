# Detailed Flow

Outlining the specific network actions, activities and things the authenticator stores at each individual step.

## Create Account

 - On successful registration the user `RootContainer` and `AccessContainer`  must be created at random locations
 - A symmetric key must be generated for encrypting those containers
 - The DataIdentifier of each, together with the must be stored in the Account on the network.
 - The root container will start out with this default set of keys, for each a random location must be chosen and a set of encryption keys generated:

```
_apps/net.maidsafe.authenticator/
_documents
_downloads
_music
_pictures
_videos
_public
_publicNames
```


## App Authorisation flow

App will send the authorisation request to the authenticator to the authorisation endpoint, `safe-auth://action:payload?riq={random-id}` as defined in the [authentication-protocol appendix](./authentication-protocol.md). Upon receiving such a request, the authenticator may grant directly access if the user has granted this app access before, or must prompt the user to grant access. Should the user grant access the authenticator must:

 1. generate a random sign key pair for the app
 2. generate a random encryption key
 3. register the public sign key with MaidManager
 4. If the app is requesting for any container access, the public sign key must added to the container along with the requested permissions
 5. If the app requested to get their own container, it must be created and full access rights granted to the public sign key
 6. if any access have been granted, a new random location must be created for the AppsAccessContainer and all container access information must be stored in there - encrypted with the newly creates apps encryption key.
 7. the application information, together with the auth information must be stored in the session packet as another `AppInfo`:

```rust
pub struct AppInfo {

  // meta information to allow a user to identify/find apps easier
  created_at: Time,
  // FIXME: options for first or always enforce to show write all even if its the same value?
  last_authenticated_at: Time, 
  last_updated_at: Time,
  // only set if the app has been revoked. Can be unset again if granted access again
  revoked: Option<Time>,

  // see authentication protocol appendix for details on those
  app_info: AppExchangeInfo,
  access_token: AppAccessToken,

  // the appsaccess container encrypted with the apps encryption key
  accessContainer: Option<DataIdentifier>
}
```


## Progressive Container Access

Any app can request access to any more containers at any point in time using the `container`-action, as defined in the authentication protocol appendix. However, the app must have been granted at least access to one other container (for example its own) prior to have received an `AppAccessContainer`. Does it not have that container yet, any progressive container requests are denied and it must do the initial authentication flow again.

Should any app do that for a container it doesn't have access to just yet - by checking the apps accessContainer using their private keys, authenticator must prompt the user to grant access. It the user grants access authenticator must:

1. add the app sign key to the container(s) access was requested for with the permissions granted
2. add the access-information (encrypted with the apps key) to its access container


## Revoking app access:

The user can, at any time, revoke an app access to their account from within the authenticator. Then the authenticator must:

 1. revoke the apps sign key from MaidManager
 2. mark the apps state as revoked in the session info
 3. clear the apps access container from all keys (aside from its own container)
 4. the authenticator may reencrypt all containers the app had access to with newly created symetric keys. If done so, it must update all other app access containers with this new key.