# Simplification of the safe-app Library API

- Status: proposed
- Type: enhancement
- Related components: `safe_app`, `safe_app_nodejs`
- Start Date: 07-03-2017
- Discussion: https://forum.safedev.org/t/simplification-of-the-mutabledata-api/480 
- Supersedes:
- Superseded by:

## Summary

This RFC proposes a refactoring on the safe-app library and the provision of a much simpler to use API to access the SAFE network, exposing it as a Rust crate while at the same time sufficing the FFI functions required for the language bindings.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

A developer can implement a client application to interact with the SAFE network either by interfacing directly with the safe_core Rust crate, or by interfacing with the safe_app library (part of the safe_client_libs) which is intended to simplify the client development process.

Based on th assumption that the safe_app library's main purpose is to aid the development of SAFE applications, the safe_app library shall expose a much simpler and easier to use API so any developer with no backgorund/knowledge of how the SAFE network internally works can quickly start creating applications for the SAFE network. 

A developer shall only need to know that data can be stored in/shared thru the network with this API, that's why functions' names must to not only be picked very carefully but they also have to be abstracted as much as possible from SAFE network internal mechanisms, functioning, and vocabulary. 

Taking the MutableData API as an example, at the moment a developer needs to handle several objects/entities for doing simple tasks like setting permissions or performing a single entry mutation, i.e. he/she needs to get references to the Permissions, PermissionsSet, Entries and/or EntryMutationTransaction objects for doing so. 

Currently, the following steps are required to create a MutableData with specific permissions:

1. Create a MutableData obj
2. Create a new PermissionsSet obj
3. Set each of the permissions in the PermissionsSet obj (with `setAllow` and `setDeny`)
3. Create a new Permissions obj
4. Insert the PermissionsSet obj (from step 2) into the Permissions obj
5. Set the Permissions obj into the MutableData and send the request to the network

The above steps shall be narrowed down to the ones below by simplifying the safe_app API:

1. Create a MutableData obj
2. Set each of the permissions by calling one of the following functions directly on the MutableData obj: `setAllow(action, signKey)` and `setDeny(action, signKey)`
3. Call a `commit` function on the MutableData obj to apply/set the permissions

An initial thought was that this could be done at the language bindings layer (e.g. in safe_app_nodejs package), but the abstractions mechanism will need to be replicated in each of the bindings. Therefore implementing this simplification at the safe_app layer will save the efforts of creating the bindings for all different entities/objects but jut the simplified API functions.


## Detailed design

### Overview

The following is a draft list the functions that the safe_app library shall expose as per this proposal.

Factory functions return a MutableData handle, whilst all the other functions expect just a MutableData handle (apart from the corresponding input parameters), i.e. no need to provide handles for the Permissions, PermissionsSet, etc..

**Factory functions:**

- `createMutableData(name, typeTag, entries)`
This function instantiates a MutableData object, sets full access permissions for the app by default, and it populates its entries.

- `createMutableData(name, typeTag)`
This function instantiates a MutableData object, sets full access permissions for the app by default, with no entries.

- `createMutableData(name, typeTag, permissions)` 
This function instantiates a MutableData object, sets the specific access permissions for the app, with no entries.


**MutableData Info:**

- `getVersion()`

- `getNameAndTag()`

**Entries functions:**

- `insert(key, value)`

- `remove(key, version)`

- `update(key, value, version)`

- `applyMutations(mutationsObj)`
It applies a bulk of mutations in a single trasaction.

- `get(key) -> (value, version)`
It returns the latest version.

- `get(key, version) -> (value, version)`
For the future, not sure how it will work when different owners had permissions for different versions.

- `getEntriesLength()`

- `forEachEntry( function(key, value, version)=>{...} )`

- `forEachKey( function(key)=>{...} )`

- `forEachValue( function(value, version)=>{...} )`

**Permissions functions:**

- `getPermissionsLength()`

- `forEachAllowPermission( function(action, signKeys)=>{...} )`

- `forEachDenyPermission( function(action, signKeys)=>{...} )`

- `setAllowPermission(action, Anyone | signKey)`

- `setDenyPermission(action, Anyone | signKey)`

- `clearPermission(action, Anyone | signKey)`

- `removeUserPermissions(Anyone | signKey)`

- `setUserPermissions(permissionsObj, Anyone | signKey)`


### Implementation Details


The following details the proposal (with flavoured pseudo code) showing how a couple of MutableData functions shall be implemented at each layer of the software stack, followed by a diagram of how the software architecture would look like.

Note that the proposed API is also exposed as a Rust crate, at the same time that sufficing the FFI required for the language bindings.

---

**A node.js application**

`safeapp.mutableData` is of type `MutableDataInterface` from the node.js binding which exposes just the static factory functions.

```
safeapp.mutableData.createPublicMutableData("mypublicId", 15001)
  .then((md) => md.insert("key1", "value1"))
  .then(() => {
    console.log("I created a MutableData and inserted entry!");
    console.log("Everything was already sent to the network.");
  })
);
```

---

**node.js binding**

```
const lib = ffi.Library("libsafe_app.so");

class MutableDataInterface {

  createPublicMutableData(name, typeTag) {  
    return lib.mdata_info_new_public(name, typeTag)
        .then((md_handle) => new MutableData(md_handle) );
  }
  ...
}

class MutableData {
  // this.md_handle holds the MutableDataHandle
  // passed at construction time.

  insert(key, value) {
    return lib.mdata_insert_entry(this.md_handle, key, value);
  }
  ...
}
```

---

**safe_app FFI**

`MutableData` is a struct in the `safe_app` crate.
`MutableDataHandle` is a type of cache handle to hold the entire MutableData data in the LRU cache.

```
pub type MutableDataHandle = ObjectHandle;

pub unsafe extern "C" fn mdata_info_new_public(name, type_tag,
                                callback: extern "C" fn(MutableDataHandle) )
{
    // Get a new MutableData object from the safe_app crate
    let md = safe_app::MutableData::new_public( name, type_tag );

    // Store the MutableData in cache
    let md_handle = object_cache.store( md );

    // Return the MutableDataHandle thru the callback function
    callback( md_handle );
};

pub unsafe extern "C" fn mdata_insert_entry(md_handle, key, value)
{
    // Fetch the MutableData from cache
    let md = object_cache.get( md_handle );

    // Insert the key-value. This invokes safe_app::MutableData.insert_entry method
    md.insert_entry(key, value);
}
```

---

**safe_app Rust crate**

It is hereby proposed to expose the API also in this layer so Rust apps can be linked directly with the crate rather than being forced to use the FFI.

```
pub struct MutableData {
    info: safe_core::MDataInfo,
}

impl MutableData {
    pub fn new_public(name, type_tag) -> MutableData {
        // Send MutableData creation request to the network
        let md_info = safe_core::MDataInfo::new_public( XorName(name), type_tag );

        // Return an instance of MutableData struct
        MutableData { info: md_info }
    }

    pub fn insert_entry(&self, key, value) {
      // Create mutation transaction
      let mut actions = Default::default();
      actions.insert('insert', key, value);

      // Send it to the network
      safe_core::client.mutate_mdata_entries(self.info.name,
                                             self.info.type_tag, actions);
    }
}
```

Note that the rest of the MutableData information shall be kept within the scope of the methods that need them, e.g. a `foreach` implementation shall be as follows:
```
    pub fn foreach_entry(&self, foreach_callback) {
        // Fetch the entries from the network and keep them in scope
        let entries = safe_core::client.list_mdata_entries(self.info.name,
                                                           self.info.type_tag)
        for entry in entries {
            foreach_callback(entry.key, entry.value);
        }
    }
```

### safe-app Software Architecture

![safe-app Software Architecture](proposed-safe-app-architecture.png)

## Drawbacks

Why should we *not* do this?

## Alternatives

What other designs have been considered? What is the impact of not doing this?


## Unresolved questions

What parts of the design are still to be done?
