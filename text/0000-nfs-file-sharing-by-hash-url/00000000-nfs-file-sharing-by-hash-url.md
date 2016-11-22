# NFS File Sharing By Hash URL 

- Status: proposed ("proposed", "agreed", "active", "implemented" or "rejected")
- Type: new feature
- Related components: SAFE Beaker Browser, safe-js, safe_core
- Start Date: 22-11-2016
- Discussion: (Preliminary to be replaced) - [File sharing at the level of safe: URLs](https://forum.safedev.org/t/file-sharing-at-the-level-of-safe-urls/288?u=happybeing) 
- Supersedes: n/a
- Superseded by: n/a

## Summary

Provide a way to obtain a "safe:" URL for any file stored using SAFE NFS that is based on the file hash (network address) and so does not involve a SAFE public ID or SAFE DNS. Such a *SAFE Hash URL* would be valid to use as part of a normal HTML page or as a file download link in SAFE Beaker Browser.

This would allow, for example, an image captured by a SAFE web app and stored in a user's SAFE NFS storage, to be loaded into the DOM of a web page/app, and displayed within by the browser as part of the HTML/app UI. 

Also, a user could request to *Share file...* and be returned a SAFE Hash URL for that file (or any file stored in their shared drive). This can be sent to anyone, who provided the file is public, could then access the image with just a SAFE web browser.  Private files could also be accessed this way but only by the authorised owner, as part of their browser history, or as quick access links in other applications or documents.

An important feature is avoiding the need for anything other than a standard SAFE Browser, such as a special file sharing app, or for the user to have made a public ID and service for the purpose of sharing files (see *Alternatives*).

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

Easy sharing of files via web links is one of the "killer" features of the current web and it would be valuable to provide this feature, while making it more functional (e.g. permanent links) and easier to use than file sharing is currently (no need to have a cloud service, no need to upload any file once stored on SAFE NFS).

Without this, the SAFE NFS user would need to use a specialist file sharing application to share a file, and possibly also to access a file shared by someone else depending on the way the sharing application is implemented.

An additional aim is to make it easy for SAFE web apps to create content, save it to the network, and to have a flexible and easy to use API for displaying it within the application. Native applications will also benefit from easier ways to share data they generate with users (e.g. as download links) or is easily sent to other applications or web apps, using a standard that every application can easily support.

Providing access to SAFE files, without requiring publication via SAFE DNS, makes it easy to port an existing app, or for someone who already knows how to create and load files on a traditional web server to integrate this functionality. Thus we can expect many apps to take advantage of these widely expected features, similar to, but even easier to use than the current web (no need to upload or use a file sharing service). It also makes it possible for this feature to implemented as permanent URLs, so that immutable data will be accessible forever from such as URL.

Example Use Cases:

1) File Sharing: users will find it useful if they can share files without first creating a public id and service. They would need to have a way of identifying the file to be shared and obtaining a URL for it, which would require an application to provide a suitable URL for a given file. This RFC recommends extending the API so that applications can do this, but doesn't consider any such applications or how they work.

2) Web Apps That Create Files: if a web app creates a file it may wish to present the file by embedding a link to it within the HTML of the app - by creating a URL for the file and embedding a link to it in the DOM. For example, an application might capture images using the webcam, store these on the network, and display them within the app by embedding image links within its HTML. To do this using SAFE NFS would be difficult, either needing a public ID and service to be created (probably one per such application). Alternatively the app have to implement a way to read files stored on SAFE NFS and render them directly on the page, which is difficult and would make it hard for the developer to take advantage of this functionality, and indeed all the advantages of doing this directly in HTML.

## Detailed design

We need a format of safe: URL which SAFE Beaker (and other compatible browsers) can recognise and use to access content stored on SAFE network without reference to a SAFE DNS public ID. Each file stored on SAFE network is addressed using a cryptographic hash, so we can use that, but we need to consider different kinds of file and how to provide expected and appropriate behaviour, depending on how the file is stored (e.g. as immutable data, mutable data), how it is identified by the user (e.g. via a directory listing obtained from SAFE NFS), and the type of the file (e.g. text, PDF, HTML, binary, video etc.).

This is tricky - to cover all cases with sensible defaults so the following is a proposal as a starting point which will probably be subject to discussion and revision:

#### SAFE Web URL Structure

A safe: URL is similar to a standard web URL, but always begins with ```safe:```, which is the link protocol which indicates a resource locator on the SAFE network. For a website published on SAFE network, we create a public ID and service using SAFE NFS, and this allows SAFE Beaker to interpret URLs of the following form:

```safe://[service.]publicid[/filepath]```

```service``` is optional, and defaults to ```www```
```filepath``` is also optional, and if service is ommitted, or is given as ```www``` it defaults to ```index.html```

So for example, the following URLs would all access the same file assuming it exists in a directory that has been published at the given service and public ID:


```safe://rfcs
safe://www.rfcs
safe://rfcs/index.html
safe://www.rfcs/index.html```


All the above is existing functionality, so the format of hash URLs must always be distinguishable from this.

### SAFE Hash URL Structure

SAFE Beaker must be able to recognise a Hash URL, parse it, and determine what method to use to access the file, and what if any meta data to provide with the HTTP response to the browser engine. We can also choose to provide a way for a human readable filename to be conveyed as part of the URL, rather than base this on the hash or require the downloading user to type it in.

Indicating a hash URL.

**A Hash URL always begins with ```safe:<type>:```** This allows us to continue to permit a normal ```safe:``` URL to omit or include the conventional but redundant '//' that we are accustomed to in web URLs.

In the following, square brackets denote an optional component. So ```[type:]hash``` means either a value denoting a ```type```, followed by a ':', followed by a value denoting a ```hash```, or just a value denoting a ```hash```.

The complete structure of a Hash URL is then.

```safe::type:hash[:filename]```

Where:

- ```type``` is one of ```file:``` or ```folder:``` or ```dns:```. So far I have only discussed retrieving regular files stored with SAFE NFS, but including a type would instruct the client on the type of data to access and how to make use of it, and could be extended to further types with backwards compatibility.

- ```hash``` is the hash address in hexadecimal which resolves to a [FILE](https://github.com/maidsafe/safe_core/blob/eb172c2718aa43b78e644f998cf65a5ff92dfa4b/src/nfs/file.rs#L29) entry

- ```filename``` is the URI encoded value of a string that must not include '/' (i.e it is a filename without a path), with an optional file extension.

Examples might be:

```safe:file:4e1243bd22c66e76c2ba9eddc1f91394e57f9f83:happybeing.png```


```safe:file:4e1243bd22c66e76c2ba9eddc1f91394e57f9f83:Safecoin-white-paper.pdf```


```safe:file:4e1243bd22c66e76c2ba9eddc1f91394e57f9f83```


This RFC does not consider the details or use of ```folder:``` or ```dns:``` or other types as these can be defined later. For now we only need implement type ```file:```.

The author doesn't adequately understand the underlying implementation of SAFE NFS storage, but believes that this method of file addressing will be feasible according to [RFC0046-new-auth-flow](https://github.com/maidsafe/rfcs/tree/master/text/0046-new-auth-flow) which uses a system of [Containers](https://github.com/maidsafe/rfcs/blob/master/text/0046-new-auth-flow/containers.md#nfs-convention) to emulate filesystem at the NFS level. This though will depend on the decisions taken around implementing emulation NFS and the underlying file structures. 

### Behaviour Options - Discussion

Disregarding any limitations imposed by the implementation we should consider the viability and usefulness SAFE Hash URLs being either permanent or temporary. Assuming the shared file is immutable, we might assume the URL would be permanent. If the file were either mutable or deliberately referenced via a mutable data structure, then the URL could be impermanent. 

At this point I'll just assume all options are on the table and open up the question of what is expected and desirable.

#### Permanent URLs


If all SAFE Hash URL's are valid forever, we have the advantage of certainty, and the feature of permanent access to all files shared in this manner.

It wouldn't ensure that SAFE websites were always available forever and immutable, because generally these contain utilise ordinary SAFE URLs that map to a mutable file structure, but the advent of *Permanent URLs* would make it *possible* to create a SAFE web app or website that was easy to inspect and verify as immutable.

One question is whether we would want to limit SAFE Hash URLs to only allow sharing of immutable data, or to allow permanent *and* impermanent URLs (see next).

A guarantee of permanence for all SAFE Hash URLs would have to be verifiable (i.e. within safe_core), because it would not be desirable for people to assume that these were permanent and immutable if there was a way of subverting this with a customised SAFE Hash URL maker).

#### Impermanent URLs

There are many good things about permanent URLs, but also the disadvantage for users who accidentally share something that can never subsequently be unshared. There are also things people want to be able to share temporarily - so as to allow the intended recipient access but later invalidate to reduce the chance that someone else later obtains the URL and gains access to the file. Also, what one shares one day, or in one state of mind, might later be regretted and so for all these situations it might also be useful to provide a way to generate URLs that can be invalidated. 

If we allow both permanent and impermanent variations it would be sensible to indicate this as part of the URL so that users know what they are sharing and receiving. In this case we might extend the type to indicate one or both variations. For example adding '-p' or '-i' to the end of the ```type``` as in ```safe:file-p:4e1243bd22c66e76c2ba9eddc1f91394e57f9f83:Safecoin-white-paper.pdf```

Impermanent URLs would require a *delete Hash URL* API, which raises a further question as to whether or not we should allow different files to appear at the same address at different times.

For simplicity's sake, this RFC mandates only permanent URLs (and so only type ```file:``` and never ```file-p``` or ```file-i```), but this assumes we can enforce permanance, and is up for debate.

### SAFE NFS API Changes

SAFE NFS API to be extended with:

```getPermanentLink(fileName: String ) -> Future<String>``` 


	```fileName```	- the full path of an existing file in the user's NFS storage


This returns a permanently valid URL for fileName formatted as a SAFE Hash URL including ```type```, ```hash``` and ```filename``` elements.

### safe-js Changes

safe-js to be extended with ```safeNFS.getPermanentLink()``` which returns a Promise that resolves to a SAFE Hash URL as above.

### SAFE Beaker Changes

SAFE Beaker will detect every SAFE Hash URL, parse it according to the above, and use this information to:

a) access the file on the network
b) provide error handling where retrieving the file fails or times out
c) deliver the content appropriately (e.g. embedded in HTML content, embedded download link, or location bar access) including where provided: applying metadata to assist in rendering embedded content, and supplying a default filename when offering to download. 

## Drawbacks

Why should we *not* do this?

## Alternatives

>>> What other designs have been considered? What is the impact of not doing this?

### Application based or shared DNS

Rather than addressing a file using a hash, we might use either the existing SAFE DNS to share files saved in the user or application managed public storage. SAFE NFS at present would be cumbersome to use for this, requiring every app which wants to generate file URLs to have access to a user's registerd public ID and service so this is not practical.

There are changes being considered which may make a DNS based sharing URLs more practical (see [this dev forum post](https://forum.safedev.org/t/file-sharing-at-the-level-of-safe-urls/288/7?u=happybeing) but these have not been decided and have not been considered as part of this RFC. So once clear, it may be worthwhile reviewing this option.

## Unresolved questions

What parts of the design are still to be done?
