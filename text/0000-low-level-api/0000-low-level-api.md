# Low Level API
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_launcher`
- Start Date: 29-August-2016

## Summary
Exposing low level API to facilitate direct construction of MaidSafe types by Launcher and apps.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
As more and more types become exposed via `safe_core` it would be impractical to provide helper functions geared towards all the use cases of various apps. It would simply lead to expolosion of such functions in the public interface. Instead if `safe_core` could provide direct interfacing with the API's somehow, then the apps/Launcher would simply call what they need and permutations would lie with them as it should be. We already have helper functions to do basic composite dns and nfs operations (like moving a file from one nfs-directory to another etc.). This RFC will deal with exposing the internals of MaidSafe types so that different or more complex operations can be done by Launcher/apps. Also representation of data as nfs-exposed directory hirearchy might not be suitable or applicable for many apps. The low level exposure would allow them to construct their own data representation of choice.

## Detailed design
Currently one of the functionality of `Launcher` is to provide sandboxing. Apps which pass through the Launcher (which is the recommended approach because it is considered bad to give one's credentials to every app) have access to data either within specific folder created for them or within `SAFEDrive` which is where common data is. No app is allowed to access data in a folder reserved for another app. However this guarantee will be broken once the low level API's are exposed because apps will have freedom to create whatever data they want and wherever they want it on the network. Under the current implementation this would mean that private data stored by one app can be potentially compromised (accessed by another app). For e.g. say App-0 creates and stores `StructuredData` `abc` somewhere in the network. If App-1 uses a direct `GET` for `abc` there is no way Launcher knows this should not be allowed. Previously apps were only allowed to travel a directory hirarchy to get data and Launcher could assert it travelled only the permissible ones.

To get around this limitation, Launcher shall enforce a rule of separate `box_::gen_keypair()` for each app. Every app that registers successfully with Launcher gets a new `box_` key-pair along with a new symmetric key-nonce pair. All private data created on the network by the app will use this key-pair to encrypt/decrypt data. These keys will need to be persistant, so Launcher will write the details in its configuration file against the registered app. Which keys will be used will be determined by `CipherOption` below.

Though the keys are persistant, there is no way for Launcher to know if one app is mimicking another during the authentication process. So Launcher will ask user each time an app starts and tries to register with Launcher, _even if Launcher was never killed in the meantime_ (unlike currently).

In the following specification, all parameters prefixed with `o_` represent _output_. Thus the base pointer must be valid. For e.g. if the type of `o_some_var_name` is `*mut *mut X` then the most base pointer (i.e. leftmost `*mut`) must be valid. Similarly for type `*mut X`, this pointer must point to a valid preallocated address.

### Cipher Options
Until now `safe_core` used owner's `Maid` keys to encrypt all private data using hybrid-encrypt scheme. This will now have to change as encryption will be governed by the choice of whether we are encrypting for ourselves or for others.

If we are encrypting for ourselves then there is no need of asymmetric/hybrid encryption as it is wasteful and doesn't enhance security. In this case we will use symmetric encryption. `AccountPacket` will contain, in addition to `Maid` asymmetric keys, a `Maid` symmetric key. This shall be used to encrypt all private user data put on the SAFEDrive unless specified otherwise, i.e. this is the default scheme.

If we are are encrypting for others, then hybrid-encryption shall be used. In this scheme, we generate a throw away symmetric key-nonce combination called `S` and encrypt the data with that. Lets call this ciphertext `C0`. Then we generate a throw away asymmetric key-pair and nonce called `asym-our and nonce-our`. We also need public part of the peer's asymmetric key. This is `Pub-asym-remote`. After this we encrypt `S` with `Sec-asym-our, nonce-our, Pub-asym-remote` and get another ciphertext `C1`. The final data now would be _Serialised(Pub-asym-our + nonce-our + C1 + C0)_. We use this technique because [asymmetric encryption is not suitable for long blocks of plaintexts](http://crypto.stackexchange.com/questions/14/how-can-i-use-asymmetric-encryption-such-as-rsa-to-encrypt-an-arbitrary-length).

```rust
#[repr(C)]
pub enum CipherOption {
    /// Data will not be encrypted.
    PlainText,
    /// Data will be encrypted using app's associated symmetric key and nonce.
    SymmetricCipher,
    /// Data will be hybrid-encrypted as described in this RFC.
    HybridCipher,
}

```

Also note that `AppHandle` in the function signatures will allow to extract app specific keys for encryption etc.

### Api's for StructuredData manipulation:
```rust
/// _type_tag_:
///   - 500 for unversioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - The actual data representation is implementation defined.
///   - 501 for versioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - `safe_core` will ensure that data is versioned.
///     - The actual data representation is implementation defined.
///   - above 15000 for user defined (no modifications from safe_core)
///   - anything else is an error
/// _id_: Pointer to array of size 32
/// _cipher_option_: If data needs to be encrypted.
/// _peer_key_: Must be valid if CipherOption::Hybrid is used. Can be null otherwise.
/// _data_: Actual data to be stored in StructuredData.
/// _size_: Size of actual data to be stored in StructuredData.
/// _o_sd_: If successful StructuredData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_create(app: *const AppHandle,
                                            type_tag: u64,
                                            id: *const [u8; 32],
                                            cipher_opt: CipherOption,
                                            peer_key: *const box_::PublicKey,
                                            data: *const u8,
                                            size: *const u64,
                                            o_sd: *mut *mut StructuredData) -> i32;


/// _data_id_: DataIdentifier for the StructuredData to be fetched.
/// _o_sd_: If successful StructuredData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get(data_id: *const DataIdentifier,
                                         o_sd: *mut *mut StructuredData) -> i32;


/// _type_tag_: type-tag of StructuredData.
/// _id_: Pointer to array of size 32.
/// _o_data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_construct_data_id(type_tag: u64,
                                                       id: *const [u8; 32],
                                                       o_data_id: *mut *mut DataIdentifier) -> i32;


/// _sd_: A valid StructuredData handle.
/// _o_data_id_: DataIdentifier of StructuredData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_extract_data_id(sd: *const StructuredData,
                                                     o_data_id: *mut *mut DataIdentifier) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 500 for unversioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - Previous data will be truncated & overwritten, hence lost.
///     - The actual data representation is implementation defined.
///   - 501 for versioned StructuredData
///     - `safe_core` will ensure that StructuredData is < 100 KiB.
///     - `safe_core` will ensure that data is versioned.
///     - New data will be added as most recent version. Previous data will not be lost.
///     - The actual data representation is implementation defined.
///   - above 15000 for user defined (no modifications from safe_core)
///     - Previous data will be truncated & overwritten, hence lost.
///   - anything else is an error
///   - StructuredData will be modified hence a mutable pointee.
/// _cipher_option_: If data needs to be encrypted.
/// _peer_key_: Must be valid if CipherOption::Hybrid is used. Can be null otherwise.
/// _data_: New data.
/// _size_: Size of new data.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_new_data(app: *const AppHandle,
                                              sd: *mut StructuredData,
                                              cipher_opt: CipherOption,
                                              peer_key: *const box_::PublicKey,
                                              data: *const u8,
                                              size: *const u64) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 500 for unversioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit.
///   - 501 for versioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit and versioned. Data pertaining to the most recent version will be
///        returned.
///   - above 15000 for user defined; there will be no modifications from safe_core - data is
///     returned as is.
///   - anything else is an error
/// _cipher_option_: If data was encrypted.
/// _o_data_: If successful the actual data will be given via this handle.
/// _o_size_: If successful the actual data size will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_data(app: *const AppHandle,
                                              sd: *const StructuredData,
                                              cipher_opt: CipherOption,
                                              o_data: *mut *mut u8,
                                              o_size: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 501 for versioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit. Data pertaining to the most recent version will be returned.
///   - anything else is an error
/// _o_versions_: If successful the number of available versions will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_num_of_versions(sd: *const StructuredData,
                                                         o_versions: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle. `type_tag` shall mean the following:
///   - 501 for versioned StructuredData
///     - `safe_core` will extract the actual data by unwraping all the implementation defined
///        covers that it created to perform various transformation to ensure data was within
///        permissible size limit and versioned.
///   - anything else is an error
/// _n_: Data pertaining to the _nth_ version will be returned.
/// _cipher_option_: If data needs to be encrypted.
/// _o_data_: If successful the actual data will be given via this handle.
/// _o_size_: If successful the actual data size will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_nth_version(app: *const AppHandle,
                                                     sd: *const StructuredData,
                                                     n: u64,
                                                     cipher_opt: CipherOption,
                                                     o_data: *mut *mut u8,
                                                     o_size: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_put(sd: *const StructuredData) -> i32;


/// _sd_: A valid StructuredData handle. `StructuredData::version` will automatically be bumped.
///       Hence it takes a mutable pointer.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_post(sd: *mut StructuredData) -> i32;

/// _sd_: A valid StructuredData handle. `StructuredData::version` will automatically be bumped.
///       Hence it takes a mutable pointer. After a call to this, it is advisable to destroy the
///       StructuredData as it will no longer exist in the SAFE Network.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_delete(sd: *mut StructuredData) -> i32;


/// _sd_: A valid StructuredData handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_free_handle(sd: *mut StructuredData) -> i32;
```

### Api's for AppendableData manipulation:
```rust
/// FFI Handle for a union of AppendableData types to reduce combinatorial explosion of functions.
pub enum AppendableDataHandle {
    Pub(PubAppendableData),
    Priv(PrivAppendableData),
}

#[repr(C)]
pub enum FilterType {
    BlackList,
    WhiteList,
}

/// _name_: Pointer to array of size 32.
/// _filter_type_: The filter criteria to be applied to supplied keys. If it's
///                `FilterType::WhiteList` and `filtered_keys` is null then nothing but owner's
///                changes will pass the filter check.
/// _filtered_keys_: Initial set of keys to apply filter to. Should be null terminated.
/// _data_: Any initial data to be stored. Will be ignored if null.
/// _o_ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_create(name: *const [u8; 32],
                                                    filter_type: FilterType,
                                                    filtered_keys: *const *const sign::PublicKey,
                                                    data: *const AppendableData,
                                                    o_ad: *mut *mut AppendableDataHandle) -> i32;


/// `PrivAppendableData::encrypt_key` will be the `box_::PublicKey` obatined for this app using
/// the supplied `AppHandle`.
/// _name_: Pointer to array of size 32.
/// _filter_type_: The filter criteria to be applied to supplied keys. If it's
///                `FilterType::WhiteList` and `filtered_keys` is null then nothing but owner's
///                changes will pass the filter check.
/// _filtered_keys_: Initial set of keys to apply filter to. Should be null terminated.
/// _data_: Any initial data to be stored. Will be ignored if null.
/// _o_ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_create(app: *const AppHandle,
                                                     name: *const [u8; 32],
                                                     filter_type: FilterType,
                                                     filtered_keys: *const *const sign::PublicKey,
                                                     data: *const AppendableData,
                                                     o_ad: *mut *mut AppendableDataHandle) -> i32;


/// _name_: Pointer to array of size 32.
/// _is_private_: If Pub/PrivAppendableData.
/// _o_data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_construct_data_id(name: *const [u8; 32],
                                                           is_private: bool,
                                                           o_data_id: *mut *mut DataIdentifier)
                                                           -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_extract_id(ad: *const AppendableDataHandle,
                                                    o_data_id: *mut *mut DataIdentifier) -> i32;


/// _data_id_: Handle to a valid DataIdentifier.
/// _o_ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get(data_id: *const DataIdentifier,
                                             o_ad: *mut *mut AppendableDataHandle) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_encrypt_key_: If successful `box_` key will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_owner_encrypt_key(ad: *const AppendableDataHandle,
                                                               o_encrypt_key: *mut *mut box_::PublicKey)
                                                               -> i32;


/// _ad_: Handle to a valid AppendableData. It will be modified, hence a mutable pointee required.
/// _filter_key_: New key to be added.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_add_filter_key(ad: *mut AppendableDataHandle,
                                                        filter_key: *const sign::PublicKey)
                                                        -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_num_: Number of AppendedData will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_list_size(ad: *const AppendableDataHandle,
                                                       o_num: *mut u64) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: DataIdentifier pertaining to the _nth_ AppendedData will be returned.
/// _o_data_id_: DataIdentifier of actual data pointed to by this AppendedData will be put into this.
///            Decryption will be performed internally if needed.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_nth_data_id(app: *const AppHandle,
                                                         ad: *const AppendableDataHandle,
                                                         n: u64,
                                                         o_data_id: *mut *mut DataIdentifier)
                                                         -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: Sign key pertaining to the _nth_ AppendedData will be returned.
/// _o_sign_key_: Sign key of this AppendedData will be put into this. Decryption will be performed
///            internally if needed.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_nth_data_sign_key(app: *const AppHandle,
                                                               ad: *const AppendableDataHandle,
                                                               n: u64,
                                                               o_sign_key: *mut *mut sign::PublicKey)
                                                               -> i32;


/// _ad_: Handle to a valid AppendableData. Will be modified, hence a mutable pointee.
/// _n_: nth AppendedData will be removed. It will be moved into the `deleted_data` field.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_delete_nth_data(ad: *mut AppendableDataHandle,
                                                         n: u64) -> i32;


/// _append_to_: Handle to a valid Pub/PrivAppendableData to which we are to append.
///                - Encryption with throw-away key etc. will all be deduced and done internally.
/// _peer_key_: Must be valid if append_to points to PrivAppendableData. Can be null otherwise.
/// _pointer_: Handle to a valid DataIdentifier (pointing to actual data) to append.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_append(app: *const AppHandle,
                                                append_to: *const DataIdentifier,
                                                peer_key: *const box_::PublicKey,
                                                pointer: *const DataIdentifier) -> i32;


/// _ad_: A valid AppendableData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_put(ad: *const AppendableDataHandle) -> i32;


/// _ad_: A valid AppendableData handle. `AppendableData::version` will automatically be bumped.
///       Hence it takes a mutable pointer.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_post(ad: *mut AppendableDataHandle) -> i32;


/// _ad_: A valid AppendableData handle. `AppendableData::version` will automatically be bumped.
///       Hence it takes a mutable pointer. After a call to this, it is advisable to destroy the
///       AppendableData as it will no longer exist in the SAFE Network.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_delete(ad: *mut AppendableDataHandle) -> i32;


/// _ad_: A valid AppendableData handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_free_handle(ad: *mut AppendableDataHandle) -> i32;
```

### Api's for ImmutableData manipulation:
```rust
/// _o_se_: New Self-Encryptor will be written to this handle.
#[no_mangle]
pub unsafe extern "C" fn immut_data_new_self_encryptor(o_se: *mut *mut SequentialEncryptor) -> i32;


/// _se_: Valid Self-Encryptor handle.
/// _data_: Raw data to be written.
/// _size_: Size of the raw data to be written.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_write_to_self_encryptor(se: *mut SequentialEncryptor,
                                                            data: *const u8,
                                                            size: u64) -> i32;


/// _se_: Valid Self-Encryptor handle. The Self-Encryptor will be destroyed after this and must not
///       be used any further. It is UB to use `se` any further.
/// _cipher_option_: If data needs to be encrypted.
/// _peer_key_: Must be valid if CipherOption::Hybrid is used. Can be null otherwise.
/// _o_data_id_: DataIdentifier of final ImmutableData will be put into this.
///              - After self-encryption, the obtained (encrypted or otherwise) data-map will be
///                tested to be <= 1 MiB. If not it will undergo self-encryption again and the
///                process repeats till we get a data-map which is <= 1 MiB. This data-map will be
///                `PUT` to the network as ImmutableData and its DataIdentifier returned.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_close_self_encryptor(app: *const AppHandle,
                                                         se: *mut SequentialEncryptor,
                                                         cipher_opt: CipherOption,
                                                         peer_key: *const box_::PublicKey,
                                                         o_data_id: *mut *mut DataIdentifier) -> i32;


/// _name_: Pointer to array of size 32.
/// _o_data_id_: DataIdentifier of ImmutableData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
pub unsafe extern "C" fn immut_data_construct_data_id(name: *const [u8; 32],
                                                      o_data_id: *mut *mut DataIdentifier) -> i32;


/// _data_id_: Valid DataIdentifier.
/// _o_se_: Self-Encryptor created from the data-map extracted via reverse process of creation as
///       detailed above. It is seamless to the app, all work being done by `safe_core`.
#[no_mangle]
pub unsafe extern "C" fn immut_data_get_self_encryptor(app: *const AppHandle,
                                                       data_id: *const DataIdentifier,
                                                       o_se: *mut *mut SequentialEncryptor) -> i32;


/// _se_: Valid Self-Encryptor handle.
/// _o_size_: Size of total available data will be written to this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_self_encryptor_size(se: *mut SequentialEncryptor,
                                                        o_size: *mut u64) -> i32;


/// _se_: Valid Self-Encryptor handle.
/// _offset_: Offset to read from.
/// _size_: Number of bytes to read from, starting from offset.
/// _o_data_: Actual data will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_read_self_encryptor(se: *mut SequentialEncryptor,
                                                        offset: u64,
                                                        size: u64,
                                                        o_data: *mut *mut u8) -> i32;


/// _ad_: A valid Self-Encryptor handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_free_self_encryptor_handle(se: *mut SequentialEncryptor) -> i32;
```

### Api's for DataIdentifier manipulation:
```rust
#[repr(C)]
enum DataType {
    Structured,
    Immutable,
    PubAppendable,
    PrivAppendable,
}

/// _data_id_: Valid DataIdentifier. Using the given handle after this is UB.
/// _o_data_type_: Deduced DataType will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn data_id_get_type(data_id: *const DataIdentifier,
                                          o_data_type: *mut DataType) -> i32;


/// _data_id_: Valid DataIdentifier. Using the given handle after this is UB.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn data_id_free_handle(data_id: *mut DataIdentifier) -> i32;
```

## Drawbacks
- If there are many apps and they are configured to work at system start-up then the user will face a barrage of pop-ups to confirm if an app should have access. This undermines the security due to inconveniencing the user.
- A few of the interfaces have parameter `peer_key` which should be valid in certain scenarios and will be ignored (hence can be nullified) in others. E.g. `appendable_data_append`, `struct_data_create` etc. This maynot be ideal api but the alternative of separating them will increase the number of functions. So balance between multiple functions vs ignorable parameters needs to be evaluated.

## Unresolved Problems
- While other apps will not be able to read the private data created by an app, they can still modify (e.g. overwrite) it, for instance if it were a `StructuredData`. To prevent this not only each app should be given its own `box_` key-pair but also `sign` key-pair. However the problem with that is only one `sign` key-pair is registered in `MaidManagers` for an account. Any other key-pair will be disallowed to create data on the network.
- The required authentication each time app wants to resume connection with Launcher means that Launcher must detect the app having gone offline. One way to do this could be a persistant connection with heartbeats but it is yet to be seen if this is a feasible approach.

## Alternatives
- Instead of passing pointers to Launcher, use a different mechanism which keeps the pointers internally and passes handle (e.g. u64) to Launcher and maps handles to pointers internally in `safe_core`, but this is better done once asyn-safe_core is flushed out.
