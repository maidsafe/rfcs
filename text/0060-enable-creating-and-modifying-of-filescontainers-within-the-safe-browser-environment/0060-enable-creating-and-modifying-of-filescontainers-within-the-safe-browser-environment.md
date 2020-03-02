# Enable creating and modifying of FilesContainers within the SAFE Browser environment

- Status: proposed
- Type: new feature / enhancement
- Related components: safe-browser, safe-api, safe-nodejs
- Start Date: 2019-02-11
- Discussion: https://safenetforum.org/t/enable-creating-and-modifying-of-filescontainers-within-the-safe-browser-environment/31088
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

## Summary

The NodeJS libraries work via the existing SAFE network CRUD conventions: we take files, we upload them to a network and generate an XORURL we can reference as a result. In the browser we do not have access to a native file sytem, so the underlying NodeJS libraries which the browser calls must allow the uploading of raw bytes, and the creation of empty FilesContainers.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

At the moment, there is no browser native way to create an empty FilesContainer (since the previous convention was the creation of an NFS container which is no longer possible).

## Detailed design

At the moment, the following API calls are available (and implemented in the [NodeJS rust library](https://github.com/maidsafe/safe-nodejs/blob/master/native/src/lib.rs)) with regards to FilesContainers:

* `files_container_create`
* `files_container_sync`
* `files_container_get`
* `files_container_add`
* `files_container_add_from_raw`

Logic would dictate that the browser would call `files_container_create` to create the files container, however the first argument to this method `location: &str` requires a path to a directory or file which will be uploaded to the FilesContainer. The same issues also exist for `files_container_sync` and `files_container_add`.

`files_container_add_from_raw` is usable, but it always defaults the `type` parameter (which is used as the HTTP `Content-Type` value in the browser to `Raw` regardless of the filetype uploaded..

I propose the modifying of 2 existing APIs in the NodeJS library, and the underlying rust SAFE-API:

* `files_container_create`
* `files_container_add_from_raw`

#### files_container_create

This method will have its first argument (`location: &str`) made optional, with the default being an empty files container.

#### files_container_add_from_raw

This method should allow the end user to set the `type` of the uploaded file.

In NodeJS pseudocode, the interface would be:

```js
function files_container_add_from_raw(
    ArrayBuffer buffer,
    string url,
    bool force,
    bool update_nrs,
    ?string type,
    ?bool dry_run
) : Array;
```

Setting a type of `text/html` should result in a downloaded `Content-Type` of `text/html`.

## Drawbacks

N/A

## Alternatives

The alternative is that FilesContainers can not be created from within the browser, which will significantly limit the abilities of SAFE browser native web apps.

## Unresolved questions

The actual code needs to be written by the Maidsafe team.
