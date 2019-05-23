# XOR-URLs (XOR-name based URLs)

- Status: proposed
- Type: new feature
- Related components: Safe Browser. Safe App Nodejs / SAFE Client libs.
- Start Date: (13-11-2018)
- Discussion: https://forum.safedev.org/t/rfc-discussion-xor-urls/2365
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

## Summary

It is herein proposed to have a resolver function in the SAFE app client API that allows us to have standardised `safe://`-URLs which are generated based on the XOR address of the content being referenced.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

Currently the SAFE app client API, and our SAFE browser, support fetching safesites and content stored on the SAFE Network with the DNS/public-names system, i.e. from URLs like `safe://<service name>.<public name>/<path>`, but it’s not possible to fetch data using their address on the network (i.e. the XOR address of any data stored on the network) without publishing it under a public name.

This represents not only a restriction in some trivial use cases, like sharing a file by sharing its location and without publishing it under a public-name, but it also prevents users from being able to store linked-data on the network as these type of data cannot be linked unless it's all published at a public-name.

For an overview of the proposal, and foresight about its potential, [this screencast video](https://www.youtube.com/watch?v=BikfxRNARnM) explores some ideas around it with a working proof of concept.

## Detailed design

### XOR address encoding
The XOR address shall be encoded in the XOR-URL, [z-base32 encoding](http://philzimmermann.com/docs/human-oriented-base-32-encoding.txt) seems to be a good choice as it is case insensitive, as opposed to other case-sensitive encodings like base58btc or base64url, and it was designed to be easier for human use permuting the alphabet so that the easier characters are the ones that occur more frequently.

### ImmutableData XOR-URL
XOR-URLs for ImmutableData’s are the simplest ones since they don’t need any additional information to uniquely identify them on the network, as opposed to MutableData’s that also have a type tag. Therefore an ImmutableData XOR-URL can be simply defined as `safe://<encoded ImmutableData XOR addr>`.

### MutableData XOR-URL
As already mentioned above, a string based on the XOR address along with a type tag is needed to uniquely identify a MutableData on the network, therefore the XOR-URL for a MD needs to include the type tag in it.

When a MutableData is fetched, if it’s not an NFS/Files container with an `index.html` file, the resolver (e.g. the `webFetch` function) can return the MutableData’s raw data so the browser (or any client application) can render it in a specific way (see below for more details of what’s being proposed here in this regard).

#### MutableData versions
MutableData’s are versioned and therefore this shall be also accountable in the XOR-URL format to allow any MD URL to (optionally) reference a specific version. This will be also necessary in the future when append-only type of data is made available on the network.

The version value can be used to enforce a specific version to be retrieved, and otherwise fail if that specific version is not found. On the other hand, the latest version will be retrieved if the version value is omitted from the XOR-URL.

#### Paths
Given that a referenced MutableData can effectively be the root NFS/Files container of the files of a safesite, any MD XOR-URL can also specify a path which needs to be resolved, just like when using the DNS/public-names system to resolve the path in a URL.

#### Browsable content
Currently, when a public-name URL is resolved to an NFS/Files container which doesn’t have an `index.html` file, the browser simply shows an error stating that the safesite content was not found.

For a MD XOR-URL that doesn’t have a path, and there is no `index.html` file or a `default` file defined (for more details about this please see refer to [this other RFC](https://github.com/joshuef/rfcs/blob/PnsAndResolveableMap/text/0000-RDF-for-public-name-resolution/0000-RDF-for-public-name-resolution.md)), the resolver can return the raw content of the MD (i.e. its key-value entries), and the browser can automatically generate an HTML page which makes the content browsable, generating links to other data when an entry’s key/value is a `safe://` string, in an analogous/similar way to how web servers on the current internet allow browsing on folders/directories.

### XOR-URLs specification
The following are the main requirements for the encoding to be used to generate the XOR-URLs:

- Be able to support new and different types of base encodings and hash functions for the XOR addresses in the future.
- Include the content type within the XOR URL which would allow the client app to correctly render the data to the user especially when referencing an ImmutableData.

It is proposed the use of [multiformats](https://github.com/multiformats) and [CID](https://github.com/ipld/cid), which allow us to cover the above requirements. A CID identifier can be used in the XOR-URL for specifying the XOR address part, and have additional parts to support MutableData’s type tag, version, as well as the path and query parts. SAFE URLs can then be defined in the following way (BNF 1-like):
```
<safe-url>            := 'safe://' ( <xor-uri> | <public-name-uri> )

<public-name-uri>     :=  [<service> '.'] <public-name> <path-query-fragment>

<xor-uri>             := <immutable-data-uri> | <mutable-data-uri>

<immutable-data-uri>  := <cid> <query-fragment>

<mutable-data-uri>    := <cid> ':' <type-tag> ['+' <content-version>] <path-query-fragment>

<path-query-fragment> := ['/' <path>] <query-fragment>

<query-fragment>      := ['?' <query>] ['#' <fragment>]
```

Where:

- `<cid>`: follows the [CID format](https://github.com/ipld/cid) which self-describes and encodes:
    - the base encoding for the string, it's here proposed to use [z-base32 encoding](http://philzimmermann.com/docs/human-oriented-base-32-encoding.txt) for the reasons explained above,
    - the version of CID for upgradability purposes (v1 now),
    - the content type or codec of the data being referenced,
    - the XOR address of the content being referenced
- `<content-version>`: for future implementation to reference versionable content, using a single address with different versions

- `<type-tag>`: the type tag value if the CID is referencing a MutableData. In the absence of this value, the CID will be assumed to be targetting an ImmutableData

- `<path>`: the path of the file if the CID is referencing a MutableData, which can be accessed through the NFS/Files convention (or other emulations/conventions in the future) and resolve it

- `<query>`: query arguments, to be used by the client app and not for retrieving the content

- `<fragment>`: fragment of the content, to be used by the client app and not for retrieving the content

The resolver function shall first attempt to decode the URL assuming it contains a `<cid>` part. If that step fails (either because it couldn’t decode the CID, or because there was no data at the decoded XOR address), it will do a fallback to assume it’s a public-name URL and it shall try to find the pulic-name MD (with the `sha3` hash of the string), just like any public-name URLs are resolved now by the resolver function.

Note that it's proposed that the resolver should try to decode the CID and attempt to fetch the content before falling back to assume it's a public-name URL, rather than only decoding. The reason behind this is to not eclipse a public-name which happens to be a valid CID string. In such cases it could still be eclipsed if there exist content at the location encoded by the CID string, but at least not do it up front when there is no content stored at such location.

The reason why the `<type-tag>` is exposed in the URL has to do with the fact that it matches quite well if you think of an analogy with ports in the clearnet, i.e. the location where the content is stored/found is the same for several MutableData's each with a different type tag (just like different services listening at different ports at the same IP address on the clearnet).

Having the `<content-version>` available to the user, allowing it to be changed in a URL to get a different version of the same content, enables the user to be explicitly aware of the version of the content it's being fetched. As mentioned above, if the user is willing to get the latest version of the targeted content then the `<content-version>` part can simply be omitted from the URL.

Since the main goal is to have a standard way of referencing and linking any piece of data stored on the SAFE Network with a URL (as required for supporting [linked-data](http://linkeddata.org/home)), thus the `<query>` and `<fragment>` parts are reserved to be used by the application consuming the content rather than used by the resolver itself. Once the resolver has located the content and fetched it, the values provided in the `<query>` and `<fragment>` parts shall be passed back to the application which requested the content. The application can then define what it's the way of referring to a specific part of the document you are targeting with the URL. E.g. the [WebID spec](https://dvcs.w3.org/hg/WebID/raw-file/tip/spec/identity-respec.html#overview) makes use of the `#fragment` to refer to one of the subgraphs, i.e. one of the persons/agents described in the WebID Profile document. As another example, the `#fragment` and `query` values are used by the web page that is being referenced by a URL and not used by the DNS to locate the file/s of the web page. In the case of the SAFE Network, the `<fragment>` could be used to refer to a specific entry of a MutableData, or the `<query>` can also be used to provide a filter to be applied to the entries that need to be considered on a referenced MutableData. This all would be defined/determined by the convention followed by the application that is fetching the content from a XOR-URL.

The following are examples of what would become valid XOR-URLs as per this proposal:

ImmutableData XOR-URL:
`safe://hygjdkfty6m7ag3bckq7eqgeizbtjk915c3jbrcgtisad8iikbk4xws4jbpky`

MutableData XOR-URL:
`safe://hyfktce8j75yhmj1dbi1xw5wnb4m3zdydr7wpbzf1a16hc3sbxzu8a9hiqw:15000`

NFS/Files container MutableData XOR-URL:
`safe://hyfktcenm57js4bm3owhez9td9pi3t8bzk1crqp7mr5865c15ih3yxpz68w:15008/some/folder/index.html`

Just as an example, given `safe://hyfktcenm57js4bm3owhez9td9pi3t8bzk1crqp7mr5865c15ih3yxpz68w:15008/some/folder/index.html#somesection?somekey=5` XOR-URL, which is referencing a MutableData which follows the convention to store hierarchy of files, the resolver will decode/decompose it in the following way to locate the content:
- `hyfktcenm57js4bm3owhez9td9pi3t8bzk1crqp7mr5865c15ih3yxpz68w` is the `<cid>` part which can be decoded by the CID spec as follows:
  - `z-base32` base encoding for the whole CID string
  - `v1` of the CID protocol/format
  - `MutableData` content type
  - `sha3-256` hash function was used to generate the XOR name/address
  - `4bdf536d057985388bfe23fb6b989c3754984737ab26cfedb25baf3207b6fe3d` XOR name/address to locate the content
- `15008` is the `<type-tag>` part, the resolver will try to fetch a MutableData with this type tag and the decoded XOR name/address
- `/some/folder/index.html` is the path which will be used by the resolver to fetch the specified file from the MutableData retrieved. This MutableData shall be a 'Files container' otherwise the resolution will fail for this case.
- `#somesection?somekey=5` is the `<fragment>` and `<query>` parts which will be ignored by the resoler and simply returned back to the application for it to use it.

### Private content

It is assumed that the URL would never contain decryption keys, not even to support sharing private content with a XOR-URL, just like the mechanism to fetch a private MutableData requires the decryption keys to be obtained from the account our of band, or with a separate mechanism not supported by the MutableData API itself. Therefore for sharing/linking private content with a XOR-URL, it could still be possible but the resolver function will need to allow to receive the decryption keys as arguments, just like the `newPrivate` function of the MutableData API, and this decryption key would need to be fetched from the account somehow by the application before invoking the resolver. At the same time, it may be considered to have either a fallback mechanism, or an additional bit encoded in the URL to realise if the XOR-URL is targeting a private or public content to act accordingly.

## Drawbacks

There is a possibility that a public-name happens to be a valid CID, and it's eclipsed by a XOR-URL with the same CID. Even that this is a drawback it should be an unlikely scenario since a public-name is meant to be for human readable URLs so having a public-name which collides with a valid CID doesn't seem to be something that can intentionally happen. Although it should be considered the case that someone may want to generate such a public-name to cheat users making them believe it's linking to an ImmutableData content when it's effectively not, or at least not until it gets eclipsed by an immutable content stored a the same location that is decoded from the CID. To prevent this possible malice events, perhaps it should be consider to have the resolver to restrict public-names in certain way, e.g. they cannot be of the exact same length as the XOR-URLs length (or length range).

## Alternatives

The alternative of not supporting this type of URLs implies there is a lack of standard URLs for referencing any content stored on the SAFE Network that can be resolved and fetched without the need to be published with the DNS/public-names system. This prevents the network from supporting the Semantic Web since linked-data is defined to make use of URIs for the links (see http://linkeddata.org/home for some definitions).

Another alternative specifically to the proposed XOR-URL format/encoding is to have a separate protocol, e.g. `safe-xor://`, for identifying these URIs. However there doesn't seem to be much benefits from doing this, and the only evident one is to avoid the scenario described above in drawback section when a public-name can be eclipsed by a XOR-URL, which it should be an unlikely scenario. On the other hand, having a single protocol as the standard for all type of URLs that resolve to content on the SAFE Network should be seen as a more homogeneous design.

## Unresolved questions

The current list of supported codecs in the [multicodec project](https://github.com/multiformats/multicodec) (which is part of the CID format that it's proposed to use) doesn't include the MIME-types that can be used for encoding in an ImmutableData XOR-URL. There is [a discussion](https://github.com/multiformats/multicodec/issues/4) which has been triggered to add such a support (see [PR sent](https://github.com/multiformats/multicodec/pull/84)) but it hasn't been approved yet.
