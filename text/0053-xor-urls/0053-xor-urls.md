# XOR-URLs (XOR-name based URLs)

- Status: proposed
- Type: new feature
- Related components: SAFE Browser, SAFE Client libs, SAFE CLI.
- Start Date: (13-11-2018)
- Discussion: https://forum.safedev.org/t/rfc-discussion-xor-urls/2365
- Supersedes: N/A
- Superseded by: N/A

## Summary

It is herein proposed to have a resolver function in the SAFE client API that allows us to have standardised `safe://`-URLs which are generated based on the XoR address of the content being referenced, as well as encoding any other piece of information required to uniquely identify the content with a URL.

In other words, such a URL (we'll call it XOR-URL from here on) would uniquely reference a single piece of native data on the SAFE Network, regardless of its type and content, which can be used by any SAFE client application to retrieve the data from the network.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

Currently the SAFE client APIs, and our SAFE browser, support fetching safesites and content stored on the SAFE Network with the NRS/public-names system, i.e. from URLs like `safe://<sub name>.<public name>/<path>` ([see RFC-0052](https://github.com/maidsafe/rfcs/blob/master/text/0052-RDF-for-public-name-resolution/0052-RDF-for-public-name-resolution.md) for more info), but it's not possible to fetch data using their address on the network (i.e. the XoR address of any data stored on the Network) without publishing it under a public name.

This represents not only a restriction in some trivial use cases, like sharing a file by sharing its location and without publishing it under a public-name, but it also means there is a lack of standard URLs for referencing any content stored on the SAFE Network, that can be resolved and fetched without the need to be published with the NRS/public-names system. This prevents the Network from supporting the Semantic Web since linked-data is defined to make use of URIs for the links (see http://linkeddata.org/home for some definitions).

For an overview of the proposal, and foresight about its potential, [this screencast video](https://www.youtube.com/watch?v=BikfxRNARnM) explores some ideas around it with a working proof of concept.

## Detailed design

### XOR-URLs specification

The following are the main requirements for the encoding format to be used to generate the XOR-URLs:

- Be able to support new and different types of base encodings and hash functions for the XOR addresses in the future.
- Include the content type within the XOR URL which would allow the client app to correctly render the data to the user, e.g. if it's an image it can launch an image viewer/editor, but if it's an audio file embed it in a module which makes it playable to the user.
- Encode the SAFE native data type where the content is stored on the Network so it can be retrieved using the corresponding, e.g. `CoinBalance`, `PublishedImmutableData`, etc.

With the introduction of XOR-URLs, as an addition to the existing NRS-URLs, we can define standard SAFE-URLs in the following way ([BNF-like](https://en.wikipedia.org/wiki/Backus%E2%80%93Naur_form)):
```
<safe-url>            ::= "safe://" (<xor-uri> | <nrs-uri>)

<nrs-uri>             ::=  *(<sub-name> ".") <public-name> <path-query-fragment>

<xor-uri>             ::= <content-id> <path-query-fragment>

<path-query-fragment> ::= *("/" <path>) [<version-and-query>] ["#" <fragment>]

<version-and-query>   ::= "?" ["v=" <content-version>] <query-args>
```

Where:

- `<content-id>` is a sequence of bytes (variable length, from 36 to 44 bytes) encoding the following information:
    - 1 byte for XOR-URL encoding format version. This is version 1
    - 2 bytes for content type, see below for the supported types and codes to be used
    - 1 byte for SAFE native data type, see below for the supported types and codes to be used
    - 32 bytes for the content's XoR name
    - 0 to 8 bytes for type tag value
    It's is here proposed to use [z-base32 encoding](http://philzimmermann.com/docs/human-oriented-base-32-encoding.txt) for the base encoding of the `content-id` string for the reasons explained in that link

- `<path>`: the path of the content, e.g. when referencing a `FilesContainer`

- `<content-version>`: the version of the content being referenced when it's versioned perpetual data. When the version is omitted from the URL, the latest available version of the content is assumed

- `<query-args>`: query arguments, to be used by the client app and not by the resolver for retrieving the content

- `<fragment>`: fragment of the content, to be used by the client app consuming the content and not for retrieving the content

The following SAFE native data types shall be supported in this first version, encoding them in the content-id with the listed codes:
- CoinBalance = 0x00
- PublishedImmutableData = 0x01
- UnpublishedImmutableData = 0x02
- SeqMutableData = 0x03
- UnseqMutableData = 0x04
- PublishedSeqAppendOnlyData = 0x05
- PublishedUnseqAppendOnlyData = 0x06
- UnpublishedSeqAppendOnlyData = 0x07
- UnpublishedUnseqAppendOnlyData = 0x08

The following list of content types shall be supported as a minimum in the first version, with the listed codes (MIME types codes shall be also allocated discretionarily by the actual implementation):
- Raw = 0x0000
- Wallet = 0x0001
- FilesContainer = 0x0002
- NrsMapContainer = 0x0003

The resolver function shall first attempt to decode the URL assuming it contains a `<content-id>` part. If that step fails, it will do a fallback to assume it's an NRS/public-name URL and it shall try to find the NRS Map Container which address shall be the `sha3` hash of the content-id string.

Having the `<content-version>` available to the user, allowing it to be changed in a URL to get a different version of the same content, enables the user to be explicitly aware of the version of the content it's being fetched. As mentioned above, if the user is willing to get the latest version of the targeted content then the content version part can simply be omitted from the URL.

Since the main goal is to have a standard way of referencing and linking any piece of data stored on the SAFE Network with a URL (as required for supporting [linked-data](http://linkeddata.org/home)), thus the `<query-args>` and `<fragment>` parts are reserved to be used by the application consuming the content rather than used by the resolver itself. Once the resolver has located the content and fetched it, the values provided in the `<query>` and `<fragment>` parts shall be passed back to the application which requested the content. The application can then define what it's the way of referring to a specific part of the document you are targeting with the URL. E.g. the [WebID spec](https://dvcs.w3.org/hg/WebID/raw-file/tip/spec/identity-respec.html#overview) makes use of the `#fragment` to refer to one of the subgraphs, i.e. one of the persons/agents described in the WebID Profile document. As another example, the `#fragment` and `query` values are used by the web page that is being referenced to and not by the NRS to locate the file/s of the web page. In the case of the SAFE Network, the `<fragment>` could be used to refer to a specific entry of a `MutableData`, or the `<query>` could be used to provide a filter to be applied to the entries on a referenced `MutableData`. This all would be defined/determined by the convention followed by the application that is fetching the content from a XOR-URL.

The following are examples of what would become valid XOR-URLs as per this proposal:

`PublishedImmutableData` XOR-URL:
`safe://hbyyyydp3xqdhobs6utkua8x48pu4c3s7b7ay1tbj7734op4r3c5zi3y4b`

`FilesContainer` XOR-URL with path to a file:
`safe://hnyynywp3xtjczdcng85986m5ixmqsqcgm4466h3pk8qk8nqn9zojthy4wbnc/some/folder/index.html`

Just as an example, given `safe://hnyynywp3xtjczdcng85986m5ixmqsqcgm4466h3pk8qk8nqn9zojthy4wbnc/some/folder/index.html?v=2&somekey=5#somesection` XOR-URL, which is referencing a `FilesContainer` which follows the convention to store hierarchy of files, the resolver will decode the URL in the following way to locate the content:
- `hnyynywp3xtjczdcng85986m5ixmqsqcgm4466h3pk8qk8nqn9zojthy4wbnc` is the `<content-id>` part which can be decoded as follows:
  - `z-base32` base encoding for the whole string
  - `v1` of the encoding format
  - `FilesContainer` deduced from the content type code
  - `PublishedSeqAppendOnlyData` deduced from the native data type code
  - `0x1b97c52cb8d8231f7f3f97babd6eb39865eb5ef732d51dca389c2fde098f01aa` XOR name/address to locate the content
  - `1100` is the type tag value
- `/some/folder/index.html?v=2` is the path and the content version (v2) which (using the information above obtained from decoding the content-id string) will be used by the resolver to fetch the specified file from version 2 of the `FilesContaier` retrieved
- `&somekey=5#somesection` is the `<query-args>` and `<fragment>` parts respectively, which will passed back to the consuming application to use it accordingly.

### Encrypted content

It is assumed that the XOR-URL would never contain/encode decryption keys to support sharing encrypted content with a XOR-URL, just like the mechanism to fetch an encrypted MutableData requires the decryption keys to be obtained out of band, or with a separate mechanism not supported by the MutableData API itself.

Therefore for sharing/linking encrypted content with a XOR-URL, it could still be possible but the resolver function will need to allow to receive the decryption key as arguments. The decryption key could be fetched from the account somehow by the application before providing it to the resolver when invoking.

## Drawbacks

There is a possibility that a public-name happens to be eclipsed by a XOR-URL. Even that this is a drawback it should be an unlikely scenario since a `public name` is meant to be for human readable URLs, thus having a public name which collides with a valid XOR-URL doesn't seem to be something that can intentionally happen. Although it should be considered the case that someone may want to generate such a public-name to cheat users making them believe it's linking to an ImmutableData content when it's effectively not, or at least not until it gets eclipsed by XOR-URL that effectively references immutable content. To prevent this possible malice events, perhaps it should be consider to have the resolver to restrict public-names in certain way, e.g. they cannot be of the exact same length as the XOR-URLs length (or length range).

## Alternatives

One alternative to the proposed encoding format is the use of [multiformats](https://github.com/multiformats) and [CID](https://github.com/ipld/cid) which has the benefit of being used by other project/s and a few of additional resources already available for it, like having libraries for several programming languages. A CID identifier can be used in the XOR-URL for specifying the XOR address part, and the [<multicodec-content-type>](https://github.com/multiformats/multicodec) part is also something we need. Although, in current version of its spec (v1) it lacks a place where we can encode the native data type as described in the proposed encoding above (i.e. `CoinBalance`, `PublishedImmutableData`, etc.). Another thing to consider is the current list of supported codecs in the [multicodec project](https://github.com/multiformats/multicodec) (which is part of the CID format) doesn't include the MIME-types that can be used for encoding in an ImmutableData XOR-URL. There is [a discussion](https://github.com/multiformats/multicodec/issues/4) which has been triggered to add such a support (see [PR sent](https://github.com/multiformats/multicodec/pull/84)) but it hasn't been approved yet.

Therefore, Multiformats CID shouldn't be a discarded alternative, but it'll need these amendments (to be proposed to Multiformats project and resolved) before we can fully benefit from it while covering our needs.

Another aspect specifically to the proposed XOR-URL format/encoding is to have a separate protocol, e.g. `safe-xor://`, for identifying these URIs. However there doesn't seem to be much benefits from doing this, and the only evident one is to avoid the scenario (described above in the [Drawbacks section](#Drawbacks)) when a public-name can be eclipsed by a XOR-URL, which should be an unlikely scenario. Having a single protocol as the standard for all type of URLs that resolve the location of the content on the SAFE Network should be seen as a more homogeneous design.
