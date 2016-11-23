# New Auth Flow

- Status: active
- Type: new feature
- Related components: launcher, safe_core, vaults
- Start Date: 04-11-2016
- Discussion: https://forum.safedev.org/t/rfc-46-new-auth-flow/294
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Appendixes: [Authentication Protocol](./authentication-protocol.md), [Containers and their basic Conventions](./containers.md), [Implementation details](./details.md)

## Summary

This RFC outlines a new process to give applications authorised access to act on the SAFE network with the user's credentials. In particular, it is designed for the mobile and embedded use case in mind.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119). The word `Account` and `session packet` will be used interchangably both refer to the `Account`-information stored through self-authentication.

## Motivation

The current authentication and permissions handling flow relies on the safe launcher, a stateful intermediate between any particular app and the network. While this allows for fine-grained control over the access flow and user setup, it has the drawback of requiring the process always to run, keep a local state and proxy all network requests. This is particularly cumbersome on embedded devices and mobile, where we cannot provide this pattern reliably.

This document proposes to replace the launcher with an alternative, stateless (if you will) approach.


## Detailed design

This is largely inspired by the [OAuth authentication flow](https://oauth.net/) as Twitter and Facebook have incorporated and made popular. These similarities should also help to explain the flow to third-party developers.

At the core stands a rethinking of the data flow and authentication responsibility. Rather than requiring every app to go through the launcher to use the users' credentials, in this approach, the user creates "sub-credentials" per app and assigns them to the app. The app can then access the network with these credentials independently from any other process running. We call these credentials `appKeys` (as they are a public-private-key-pair attached to a users maidsafe keys), which an app can interact with the network directly with.

This requires the way authentication flow is handled and how the corresponding parties behave. In particular, this RFC proposes to remove launcher entirely. In its place, we want to provide a cross-platform "Authenticator" App (see "shipping authenticator" below, for more about this), which should become the new entry point for the user to the network and the only place to manage authenticated apps. However, this app will no longer bridge outside requests to the network but only interact with the user when an app asks for permissions.

Apps will have to directly interact with the network. We will restructure `safe_core` to provide all needed facility for that, as well as the proposed authentication flow against the authenticator. That means that each App has to bundle their own `safe_core` and link to that library. The library will expose all necessary functionality also through FFI to allow for binding in other environments than rust.

### Storing App Permissions

Permissions granted to the applications are stored in the user's session packet and thus it is not accessible for applications, but can be retrieved and handed out by the authenticator at any point in time.

## Application Authorisation

The applications invoke the authenticator for authorisation using a URL scheme. The data exchange between application and authenticator happen via a custom protocol. The details can be found in Appendix A: The Authentication Protocol.

Through this protocol, the app may gain access to its app container and any of the shared resource the user has (see Appendix B: Containers), given that the User granted access to them. The authentication keys the authenticator returns are persistent and can be used to connect directly to the network in the user's name without any other third party running. Thus the app can connect to the network independently.


### Authorisation flow overview

- When an App asks to access the user's area, the Authenticator must prompt the user about granted access as requested. This may be a multi-step process if the App request unusual permissions, but should otherwise be a one-click procedure.
- If the App has not accessed the user's data before, the Authenticator must create a new random-key-pair for the App, store its sign key in the users Session Paket, which lets vaults know the key is valid to sign in the user's name. It then creates a new container for the app at another random location.
- The user's sign key and app's sign key are added to the permissions of said container with full access rights and returned to the app
- The app can now access the network and those resources directly with the given credentials.

Please refer to the appendix about the authentication protocol and how containers work to get an in-depth understanding.


Further notes:

- As all tokens are persistent, it is the apps responsibility to ensure their safety. If these cannot be guaranteed, the app should only keep them in memory (or even safely discard then if possible) and request the same permissions again on the next startup.
- Upon receiving permission requests of access level already authorised to an app, the Authenticator should respond with those without any further user interaction immediately.
- The authorisation flow contains a specific `scope` field, allowing an app to let the authenticator know that this is certain sub-part (a specific website, specific device or instance) is trying to access. App keys and containers are scoped using this field and any request without said field will be granted access to all under that scope.
- This mechanism should be used by any app that requests access _on behalf_ of another instance, e.g. any web browser (where the scope should be the URL) or a tool that registers IoT/Embedded/headless devices with the authenticator.


#### Revoking General Access For An App

Please refer to the container appendix to learn what happens when the User revokes access to certain shared resources. Furthermore, the user may instruct the Authenticator to remove the complete access of an App.

The Authenticator then removes the apps key from the user session, restricting the app from acting in the user's name. Secondly, it removes the app keys from the apps container and any other container it may have shared access to as defined in the Containers Appendix. However, the Authenticator SHOULD keep the key-pair cached to allow the user to grant the same key-pair again, as well as use it later in time to claim ownership of objects the app might have created.


## Technical Restructure

With apps directly connecting to the network, we will clean up `safe_core` and reduce its features to compass only what is common between authenticator and apps. Its main entry-point being a client of the following structure:

```rust
// FIXME: this needs refinement!
pub struct Client {
  // all of them are most likely asynchronous later
  fn log_in(locator, password) -> Result<Client, Error>;
  fn create_account(locator, password)-> Result<Client, Error>;
  fn get_unregistered_client() -> Result<Client, Error>;

  fn get_access_container(&self) -> Result<DataIdentifier, Error>;

  // List the authorised apps from session packet
  fn get_apps(&self) -> Vec<AppInfo>;

  fn transfer_ownership(&self, mutable_data, toPublicSignKey) -> Result<(), Error>;

  // Should handle complete revoke workflow
  fn revoke_app(&self, access_token: AppAccessToken) -> Result<(), Error>;

  // For Apps
  fn connect(access_token: AppAccessToken, config: Vec<u8>) -> Result<Client, Error>;
  fn get_public_maid_sign_key(&self) -> Result<PubSignKey, Error>;

  // ..other Data type APIs
  Mutable Data, Immutable Data (Self Encrypt)
}
```

Everything authenticator specific (like app management facilities) will move into a newly created `safe_authenticator`-rust project, which depends on `safe_core`, which will become the foundation for the newly created Authenticator tool. Learn more about this in the "shipping authenticator" below.

Secondly, we will produce a `safe_sdk` for app development that creates all higher feature set that any apps might need: like NFS and other conventions. It will use `safe_core` to connect to the network but also feature a full implementation of the `safe_auth`-protocol including protocol-scheme-registration with the system. This crate will include an FFI interface for non-rust development environments. Some specific features might be split up into separate crates where appropriate.

### Mobile Infrastructure

The project will provide SDKs for iOS and Android, which will implement the authentication protocol and provide a native interface of `safe_sdk` to app developers through FFI. All platforms will use the very same _rust_ core libraries.

### Building and Shipping Authenticator

All heavy lifting of the authenticator should be performed in cross-platform rust library `safe_authenticator`, while the management UI will be written in Javascript/React (HTML and CSS). The Javascript app will perform all interaction of the authenticator library through a DOM-like FFI-interface.

We intend to use the same code base (both rust and JS/HTML) on iOS, Android, Desktop as an independently running process as well as an extension for the SAFE browser. This will be achieved through the cross-builder meta-project which allows us to bundle the app for mobile using Cordova, for Desktop using electron and as a standalone package for the SAFE browser with one command. It will include all JS and Rust code and compiles them according to the needs of the target platform or all at once.

Further, it will contain all integrations needed to expose the Rust library to the Javascript layer the same way on all platforms, as well as manage the safe-auth-protocol (like schema registration).


## Drawbacks

- Though the use case should be vastly reduced, application impersonating another application is possible. However, should this become a problem the authentication protocol designed to allow for extended measures (like public-key-signing).
- Apps have to bundle a native library now
- Without a launcher intermediate, we cannot offer the same depth of introspection what apps are up to and graphs about this for the moment. While we will add this to `safe_core` in the mid-term, we can not guarantee that someone might disable this feature to hide activity.

## Alternatives

None
