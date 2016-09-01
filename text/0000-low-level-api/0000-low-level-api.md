# Low Level API
- Status: proposed
- Type: feature
- Related components: `safe_core`, `safe_launcher`
- Start Date: 29-August-2016

## Summary
Exposing low level API to facilitate direct construction of MaidSafe types by Launcher and apps.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).
- All parameters prefixed with `o_` represent _output_. Thus the base pointer must be valid. For e.g. if the type of `o_some_var_name` is `*mut *mut X` then the most base pointer (i.e. leftmost `*mut`) must be valid. Similarly for type `*mut X`, this pointer must point to a valid preallocated address.

## Motivation
As more and more types become exposed via `safe_core` it would be impractical to provide helper functions geared towards all the use cases of various apps. It would simply lead to expolosion of such functions in the public interface. Instead if `safe_core` could provide direct interfacing with the API's somehow, then the apps/Launcher would simply call what they need and permutations would lie with them as it should be. We already have helper functions to do basic composite dns and nfs operations (like moving a file from one nfs-directory to another etc.). This RFC will deal with exposing the internals of MaidSafe types so that different or more complex operations can be done by Launcher/apps. Also representation of data as nfs-exposed directory hirearchy might not be suitable or applicable for many apps. The low level exposure would allow them to construct their own data representation of choice.

## Detailed design
Currently one of the functionality of `Launcher` is to provide sandboxing. Apps which pass through the Launcher (which is the recommended approach because it is considered bad to give one's credentials to every app) have access to data either within specific folder created for them or within `SAFEDrive` which is where common data is. No app is allowed to access data in a folder reserved for another app. However this guarantee will be broken once the low level API's are exposed because apps will have freedom to create whatever data they want and wherever they want it on the network. Under the current implementation this would mean that private data stored by one app can be potentially compromised (accessed by another app). For e.g. say App-0 creates and stores `StructuredData` `abc` somewhere in the network. If App-1 uses a direct `GET` for `abc` there is no way Launcher knows this should not be allowed. Previously apps were only allowed to travel a directory hirarchy to get data and Launcher could assert it travelled only the permissible ones.

To get around this limitation, Launcher shall enforce a rule of separate `secretbox::gen_key()` for each app that registers successfully with it. All private data created on the network by the app will use this to encrypt/decrypt data. These keys will need to be persistant, so Launcher will write the details in its configuration file against the registered app. Which keys will be used will be determined by `CipherOption` below. Note that, new nonce shall be generated everytime encryption is used. The final data shall look like:
```
Serialised(nonce + C0) // where C0 is the cipher-text
```
If we are are encrypting for others, we generate a throw away asymmetric key-pair and nonce called `asym-our and nonce-our`. We also need public part of the peer's asymmetric key. This is `pub-asym-remote`. After this we encrypt plaintext with `sec-asym-our, nonce-our, pub-asym-remote` and get ciphertext `C0`. The final data now would be :
```
Serialised(pub-asym-our + nonce-our + C0)
```
We use this technique because asymmetric encryption using _Curve25519_ is suitable for any length block of plaintexts as is evident from [here](http://crypto.stackexchange.com/a/29332/27866) and [here (under _High-level primitives_)](https://nacl.cr.yp.to/features.html).

Though the keys are persistant, there is no way for Launcher to know if one app is mimicking another during the authentication process. So Launcher will ask user each time an app starts and tries to register with Launcher, _even if Launcher was never killed in the meantime_ (unlike currently).

### Change in safe_core::core
Until now `safe_core` used owner's `Maid` asymmetric-keys to encrypt all private data using hybrid-encrypt scheme. To be in line with the above changes this will now have to change as encryption will be governed by the choice of whether we are encrypting for ourselves or for others.  If we are encrypting for ourselves then there is no need of asymmetric/hybrid encryption as it is wasteful and doesn't enhance security. In this case we will use symmetric encryption.

### Cipher Options
```rust
pub enum CipherOption {
    /// Data will not be encrypted.
    PlainText,
    /// Data will be encrypted using app's associated symmetric key and nonce.
    Symmetric,
    /// Data will be encrypted using app's associated asymmetric key and nonce and peer's PublicKey.
    Asymmetric {
        peer_encrypt_key: box_::PublicKey,
    },
}
```

### Choice of API
`safe_core`'s use-case is very different form conventional libraries. Usually one would have the frontend interfacing with the library (dynamic or static) directly resulting in an executable. In these cases we go for the standard FFI interface where many opaque-pointer-handles are exposed via the libary. For instance consider there is a function to obtain handle to an opaque object, a function to manipulate it and a function to destroy it. It would look something standard like:
```rust
// Allocation of the pointer to pointer done by NodeJS.
// Allocation of final pointer done by `safe_core`.
#[no_mangle]
pub unsafe extern "C" fn create(handle: *mut *mut Opaque) -> i32;

// UB if invalid handle - hence unsafe.
#[no_mangle]
pub unsafe extern "C" fn manipulate(handle: *mut Opaque) -> i32;

// UB if invalid handle - hence unsafe.
#[no_mangle]
pub unsafe extern "C" fn destroy(handle: *mut Opaque) -> i32;
```
However the case here is different - the apps would want the same functionality but are not binary-interfaced with `safe_core`. The are completely separate processes and would talk through RPCs. In such case `safe_core` can avoid passing opaque pointer handles to Launcher and manage it internally itself. This would make interfaces lot safer (far fewer chances Undefined Behaviours). In the present API choice, `safe_core` maintains an LRU-based object cache and handles are returned as `u64`. The interfaces now change to:
```rust
// Allocation entirely done by NodeJS.
#[no_mangle]
pub unsafe extern "C" fn create(handle: *mut u64) -> i32;

// No chance of UB - will return error at most if mapping is absent. Note no `unsafe` mark.
#[no_mangle]
pub extern "C" fn manipulate(handle: u64) -> i32;

// No chance of UB - will return error at most if mapping is absent. Note no `unsafe` mark.
#[no_mangle]
pub extern "C" fn destroy(handle: u64) -> i32;

// Internally in `safe_core`
type OpaqueCache = LruCache<u64, (Option<AppHandle>, Opaque)>;
```
Notice how majority of operations is now in safe-rust. Further, we use `LruCache` so that if a misbehaving app does not free resource (does not call `destroy()`), we make sure there is no indefinite memory leak as `LruCache` will clean up unused objects once its capacity is filled. Storing `AppHandle` in `Option<AppHandle>` will give us nice statistis and profiling to find apps that leave maximum amount of uncleaned objects when memory footprint starts growing and we make a scan of our object-cache.

We will use the following typedef for such a handle:
```rust
pub type ObjectHandle = u64;
```

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
pub unsafe extern "C" fn struct_data_create(app: ObjectHandle,
                                            type_tag: u64,
                                            id: *const [u8; 32],
                                            cipher_opt: ObjectHandle,
                                            data: *const u8,
                                            size: u64,
                                            o_sd: *mut ObjectHandle) -> i32;


/// _data_id_: DataIdentifier for the StructuredData to be fetched.
/// _o_sd_: If successful StructuredData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_get(data_id: ObjectHandle,
                                         o_sd: *mut ObjectHandle) -> i32;


/// _type_tag_: type-tag of StructuredData.
/// _id_: Pointer to array of size 32.
/// _o_data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_construct_data_id(type_tag: u64,
                                                       id: *const [u8; 32],
                                                       o_data_id: *mut ObjectHandle) -> i32;


/// _sd_: A valid StructuredData handle.
/// _o_data_id_: DataIdentifier of StructuredData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_extract_data_id(sd: *const StructuredData,
                                                     o_data_id: *mut ObjectHandle) -> i32;


/// _sd_: A valid StructuredData handle.
/// _o_owner_: Boolean result will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn struct_data_assert_ownership(sd: ObjectHandle, o_owner: *mut bool) -> i32;


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
pub unsafe extern "C" fn struct_data_new_data(app: ObjectHandle,
                                              sd: ObjectHandle,
                                              cipher_opt: ObjectHandle,
                                              peer_key: ObjectHandle,
                                              data: *const u8,
                                              size: u64) -> i32;


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
pub unsafe extern "C" fn struct_data_get_data(app: ObjectHandle,
                                              sd: ObjectHandle,
                                              cipher_opt: ObjectHandle,
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
pub unsafe extern "C" fn struct_data_get_num_of_versions(sd: ObjectHandle,
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
pub unsafe extern "C" fn struct_data_get_nth_version(app: ObjectHandle,
                                                     sd: ObjectHandle,
                                                     n: u64,
                                                     cipher_opt: ObjectHandle,
                                                     o_data: *mut *mut u8,
                                                     o_size: *mut u64) -> i32;


/// _sd_: A valid StructuredData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn struct_data_put(sd: ObjectHandle) -> i32;


/// _sd_: A valid StructuredData handle. `StructuredData::version` will automatically be bumped.
///       Hence it takes a mutable pointer.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn struct_data_post(sd: ObjectHandle) -> i32;

/// _sd_: A valid StructuredData handle. `StructuredData::version` will automatically be bumped.
///       Hence it takes a mutable pointer. After a call to this, it is advisable to destroy the
///       StructuredData as it will no longer exist in the SAFE Network.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn struct_data_delete(sd: ObjectHandle) -> i32;


/// _sd_: A valid StructuredData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn struct_data_free_handle(sd: ObjectHandle) -> i32;
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
/// _data_: Any initial data to be stored. Will be ignored if null.
/// _o_ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn pub_appendable_data_create(name: *const [u8; 32],
                                                    filter_type: FilterType,
                                                    data: *const u8,
                                                    size: u64,
                                                    o_ad: *mut ObjectHandle) -> i32;


/// `PrivAppendableData::encrypt_key` will be the `box_::PublicKey` obatined for this app using
/// the supplied `AppHandle`.
/// _name_: Pointer to array of size 32.
/// _filter_type_: The filter criteria to be applied to supplied keys. If it's
///                `FilterType::WhiteList` and `filtered_keys` is null then nothing but owner's
///                changes will pass the filter check.
/// _data_: Any initial data to be stored. Will be ignored if null.
/// _o_ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn priv_appendable_data_create(app: ObjectHandle,
                                                     name: *const [u8; 32],
                                                     filter_type: FilterType,
                                                     data: *const u8,
                                                     size: u64,
                                                     o_ad: *mut ObjectHandle) -> i32;


/// _name_: Pointer to array of size 32.
/// _is_private_: If Pub/PrivAppendableData.
/// _o_data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_construct_data_id(name: *const [u8; 32],
                                                           is_private: bool,
                                                           o_data_id: *mut ObjectHandle)
                                                           -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_filter_type_: If successful FilterType will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_filter_type(ad: ObjectHandle,
                                                         o_filter_type: *mut FilterType) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _filter_type_: New FilterType.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_change_filter_type(ad: ObjectHandle,
                                                            filter_type: FilterType) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_data_id_: If successful DataIdentifier will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_extract_id(ad: ObjectHandle,
                                                    o_data_id: *mut ObjectHandle) -> i32;


/// _data_id_: Handle to a valid DataIdentifier.
/// _o_ad_: If successful AppendableData will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get(data_id: ObjectHandle,
                                             o_ad: *mut ObjectHandle) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_encrypt_key_: If successful `box_` key will be given via this handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_owner_encrypt_key(ad: ObjectHandle,
                                                               o_encrypt_key: *mut ObjectHandle)
                                                               -> i32;


/// _ad_: Handle to a valid AppendableData. It will be modified, hence a mutable pointee required.
/// _filter_key_: New key to be added.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_add_filter_key(ad: ObjectHandle,
                                                 filter_key: ObjectHandle)
                                                 -> i32;


/// _ad_: Handle to a valid AppendableData. It will be modified, hence a mutable pointee required.
/// _filter_key_: New key to be added.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_remove_filter_key(ad: ObjectHandle,
                                                    filter_key: ObjectHandle)
                                                    -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _o_size_: Number of AppendedData will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_list_size(ad: ObjectHandle,
                                                       o_size: *mut u64) -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: DataIdentifier pertaining to the _nth_ AppendedData will be returned.
/// _o_data_id_: DataIdentifier of actual data pointed to by this AppendedData will be put into this.
///            Decryption will be performed internally if needed.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_nth_data_id(app: ObjectHandle,
                                                         ad: ObjectHandle,
                                                         n: u64,
                                                         o_data_id: *mut ObjectHandle)
                                                         -> i32;


/// _ad_: Handle to a valid AppendableData.
/// _n_: Sign key pertaining to the _nth_ AppendedData will be returned.
/// _o_sign_key_: Sign key of this AppendedData will be put into this. Decryption will be performed
///            internally if needed.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn appendable_data_get_nth_data_sign_key(app: ObjectHandle,
                                                               ad: ObjectHandle,
                                                               n: u64,
                                                               o_sign_key: *mut ObjectHandle)
                                                               -> i32;


/// _ad_: Handle to a valid AppendableData. Will be modified, hence a mutable pointee.
/// _n_: nth AppendedData will be removed. It will be moved into the `deleted_data` field.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_delete_nth_data(ad: ObjectHandle, n: u64) -> i32;


/// _append_to_: Handle to a valid Pub/PrivAppendableData to which we are to append.
///                - Encryption with throw-away key etc. will all be deduced and done internally.
/// _peer_key_: Must be valid if append_to points to PrivAppendableData. Can be null otherwise.
/// _pointer_: Handle to a valid DataIdentifier (pointing to actual data) to append.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_append(app: ObjectHandle,
                                         append_to: ObjectHandle,
                                         peer_key: ObjectHandle,
                                         pointer: ObjectHandle) -> i32;


/// _ad_: A valid AppendableData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_put(ad: ObjectHandle) -> i32;


/// _ad_: A valid AppendableData handle. `AppendableData::version` will automatically be bumped.
///       Hence it takes a mutable pointer.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_post(ad: ObjectHandle) -> i32;


/// _ad_: A valid AppendableData handle. `AppendableData::version` will automatically be bumped.
///       Hence it takes a mutable pointer. After a call to this, it is advisable to destroy the
///       AppendableData as it will no longer exist in the SAFE Network.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_delete(ad: ObjectHandle) -> i32;


/// _ad_: A valid AppendableData handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn appendable_data_free_handle(ad: ObjectHandle) -> i32;
```

### Api's for ImmutableData manipulation:
```rust
/// _o_se_: New Sequential-Encryptor will be written to this handle.
#[no_mangle]
pub unsafe extern "C" fn immut_data_new_self_encryptor(o_se: *mut ObjectHandle) -> i32;


/// _se_: Valid Sequential-Encryptor handle.
/// _data_: Raw data to be written.
/// _size_: Size of the raw data to be written.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_write_to_self_encryptor(se: ObjectHandle,
                                                            data: *const u8,
                                                            size: u64) -> i32;


/// _se_: Valid Sequential-Encryptor handle. The Sequential-Encryptor will be destroyed after this
///       and must not be used any further.
/// _cipher_option_: If data needs to be encrypted.
/// _peer_key_: Must be valid if CipherOption::Hybrid is used. Can be null otherwise.
/// _o_data_id_: DataIdentifier of final ImmutableData will be put into this.
///              - After self-encryption, the obtained (encrypted or otherwise) data-map will be
///                tested to be <= 1 MiB. If not it will undergo self-encryption again and the
///                process repeats till we get a data-map which is <= 1 MiB. This data-map will be
///                `PUT` to the network as ImmutableData and its DataIdentifier returned.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_close_self_encryptor(app: ObjectHandle,
                                                         se: ObjectHandle,
                                                         cipher_opt: ObjectHandle,
                                                         o_data_id: *mut ObjectHandle) -> i32;


/// _name_: Pointer to array of size 32.
/// _o_data_id_: DataIdentifier of ImmutableData will be put into this.
/// **return-value**: Non-zero in case of error giving the error-code.
pub unsafe extern "C" fn immut_data_construct_data_id(name: *const [u8; 32],
                                                      o_data_id: *mut ObjectHandle) -> i32;


/// _data_id_: Valid DataIdentifier.
/// _o_se_: Sequential-Encryptor created from the data-map extracted via reverse process of creation as
///       detailed above. It is seamless to the app, all work being done by `safe_core`.
#[no_mangle]
pub unsafe extern "C" fn immut_data_get_self_encryptor(app: ObjectHandle,
                                                       data_id: ObjectHandle,
                                                       o_se: *mut ObjectHandle) -> i32;


/// _se_: Valid Sequential-Encryptor handle.
/// _o_size_: Size of total available data will be written to this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_self_encryptor_size(se: ObjectHandle, o_size: *mut u64) -> i32;


/// _se_: Valid Sequential-Encryptor handle.
/// _offset_: Offset to read from.
/// _size_: Number of bytes to read from, starting from offset.
/// _o_data_: Actual data will be written into this.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_read_self_encryptor(se: ObjectHandle,
                                                        offset: u64,
                                                        size: u64,
                                                        o_data: *mut *mut u8) -> i32;


/// _se_: A valid Sequential-Encryptor handle.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn immut_data_free_self_encryptor_handle(se: ObjectHandle) -> i32;
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
pub unsafe extern "C" fn data_id_get_type(data_id: ObjectHandle,
                                          o_data_type: *mut DataType) -> i32;


/// _data_id_: Valid DataIdentifier.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub extern "C" fn data_id_free_handle(data_id: ObjectHandle) -> i32;
```

### Implementation
A small twist can greatly make the API more readable. Instead of using `ObjectHandle` everywhere we could make API more self-explicable by havig multiple of these:
```rust
pub type AppHandle = ObjectHandle;
pub type StructDataHandle = ObjectHandle;
pub type AppendableDataHandle = ObjectHandle;
pub type DataIdHandle = ObjectHandle;
pub type SequentialEncryptorHandle = ObjectHandle;
pub type CipherOptHandle = ObjectHandle;
pub type EncryptKeyHandle = ObjectHandle;
pub type SignKeyHandle = ObjectHandle;
```
and then change signatures from:
```rust
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_data(app: ObjectHandle,
                                              sd: ObjectHandle,
                                              cipher_opt: ObjectHandle,
                                              o_data: *mut *mut u8,
                                              o_size: *mut u64) -> i32;
```
to:
```rust
#[no_mangle]
pub unsafe extern "C" fn struct_data_get_data(app: AppHandle,
                                              sd: StructDataHandle,
                                              cipher_opt: CipherOptHandle,
                                              o_data: *mut *mut u8,
                                              o_size: *mut u64) -> i32;
```
and so on - a slight increase in readability because we are not just relying on the parameter-name (variable-name) to get the handle right but also explicitly specifying as a type, so less chances of reading the interface wrongly by app-devs.

## Drawbacks
- If there are many apps and they are configured to work at system start-up then the user will face a barrage of pop-ups to confirm if an app should have access. This undermines the security due to inconveniencing the user.

## Future scope
- While other apps will not be able to read the private data created by an app, they can still modify (e.g. overwrite) it, for instance if it were a `StructuredData`. To prevent this not only each app should be given its own `box_` key-pair but also `sign` key-pair. However the problem with that is only one `sign` key-pair is registered in `MaidManagers` for an account. Any other key-pair will be disallowed to create data on the network. The are approaches to this problem and is considered future scope right now.

## Alternatives
### FFI interface with Opaque Pointers.
In this case, opaque pointers will be returned to `Launcher` for it to manage (keep alive, not alias with something else etc.). For e.g. `SequentialEncryptor` API would look like this instead.
```rust
/// _o_se_: New Sequential-Encryptor will be written to this handle.
#[no_mangle]
pub unsafe extern "C" fn immut_data_new_self_encryptor(o_se: *mut *mut SequentialEncryptor) -> i32;


/// _se_: A valid Sequential-Encryptor handle. After a call to this, it is UB to use the handle any more.
/// **return-value**: Non-zero in case of error giving the error-code.
#[no_mangle]
pub unsafe extern "C" fn immut_data_free_self_encryptor_handle(se: *mut SequentialEncryptor) -> i32;
// Note this ^^^ function is `unsafe` now.
```

This has the disadvantage as explained in this RFC above under design choice topic.
