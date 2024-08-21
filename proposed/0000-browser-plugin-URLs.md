- Feature Name: Browser Plugin URL Handling
- Type: New
- Related: SAFE Browser Plugin
- Start Date: 07-08-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

A method of URL handling and an extension to the SAFE Browser Plugin API, that will allow dynamic HTML pages to be supported in an efficient manner, entirely on the client side. It includes a method to GET the client side HTML/Javascript document concurrently with GET of other documents needed to render the page, to speed up page loading. Also for within-page bookmarking using fragment identifier.

# Motivation

This functionality is required in the plugin so that the client side can do things that are normally handled on the server when generating HTML pages dynamically. 

The reason is that a "safe:" URL (SAFE URL) such as "safe:happybeing/somewebpage" can only deliver one document to the client. This would work if every page was stored as a separate HTML document, but does not provide for generating pages dynamically. 

Example: we have a blog and want to show a post. There is a "post template" that defines how the page is to be laid out in HTML, with placeholders for the content of the blog post itself. With server side processing, an HTTP request includes all the information needed for the server to identify the template and the post, retrieve both, and generate a merged document which it returns and is loaded into the browser as a single file.

We need a way to allow equivalent expression of "template" and "content" within a SAFE URL, and for this to be processed efficiently. To make this efficient, we can't just fetch the template, load it, and suffer delays while the client side issues additional GET requests for the content. So this RFC proposes that we design the plugin to decode a single URL into component URLs which identify the page to be loaded (i.e. template) and any additional content URLs, and to fetch all these documents concurrently. The page loaded by the browser will then be able to include client side Javascript to merge the remaining content as it is received.

Including support for a [fragment identifier](https://en.wikipedia.org/wiki/Fragment_identifier) means that SAFE URLs and bookmarks can refer to a location *within* a page.

With support in the plugin for these features, SAFE URLs can be bookmarked, shared etc. by users and sharing tools just like existing browser addresses, and exist seamlessly alongside exsiting internet addresses.

For more information on URL design and the terminology used in this RFC please see the following resource:

- [URL Syntax (Wikipedia)](https://en.wikipedia.org/wiki/Uniform_resource_locator#Syntax)

# Detailed design


## Plugin URL Handling Process

The SAFE Browser Plugin (plugin) modifies the browser handling for URLs using the **scheme** we've chosen for SAFE network access, which is any URL starting with "safe:" (i.e. a SAFE URL). This enables the plugin to perform special handling of URLs directed at the SAFE Network seamlessly alongside other URL schemes such as "http:", "ftp:" etc. Typically this might just be to retrieve a document stored on SAFE rather than stored on a server accessed via the internet.

### SAFE URL Fragment Identifier

Permitting a [fragment identifier](https://en.wikipedia.org/wiki/Fragment_identifier) allows URLs (e.g. bookmarks) to refer to a position within the content of an HTML page, by indicating the HTML id of the element which is to be made visible by the browser in its viewport. To support this, the plugin must remove any fragment identifier from the SAFE URL, or the GET would attempt to retrieve the wrong document.

On intercepting a SAFE URL, the plugin will scan it for an HTML fragment identifier ("#") and strip this from the URL, storing the text after the identifier for access via the API by reference to a reserved variable name (e.g. "_FRAGMENT").

### SAFE URL Query String

The plugin will recognise and process a [query_string](https://en.wikipedia.org/wiki/Query_string) such as *"?first_name=John&last_name=Doe"* appearing within a SAFE URL.

After stripping any fragment identifier, the plugin will scan for a **query_string**, and strip this from the URL before making the GET request for the content. After initiating the GET request on the stripped SAFE URL it will process the query string as follows.

Each "parameter=value" pairing will be decoded, and stored as a name/value pair in an object that is accessible to the client Javascript loaded once the response to the GET has been returned to the browser for loading.

Where the "value" begins with the sequence "GET:safe:" and is followed by additional characters, the "GET:" will be removed and the remainder treated as a SAFE URL, and a new GET issued to retrieve it. Once the data has been received for these requests, or an error raised about them, this will be accessible to the client Javascript via the plugin API. The client Javascript can then merge content into the HTML of the page as it is received.

As an example, a SAFE URL including a query string to retrieve and display a blog post might look like this:

```
safe:happybeing/blog?post=GET:safe:happybeing/posts/safe-network-the-secure-decentralised-internet#features
```

This would be broken down into:

- "safe:happybeing/blog" - the SAFE URL to be loaded into the browser, accessible as a specially named variable (e.g. "_DOCUMENT_URL")
- "safe:happybeing/posts/safe-network-the-secure-decentralised-internet" - an additional SAFE URL to be retrieved and made accessible via the API (accessible as the value of "post")
- "features" - an HTML anchor reference, accessible as a specially named variable (e.g. "_FRAGMENT_IDENTIFIER")

There may be cases where it is useful to have multiple "GET:safe:" instructions within a query string, all retrieved concurrently, and so the intention is that there be no arbitrary limit on how many are permitted within a single SAFE URL.

#### Reserved SAFE URL Characters

To ensure that all possible SAFE URLs can be processed I propose that "?", "&", "#" and "%" be [reserved characters](https://en.wikipedia.org/wiki/Uniform_resource_locator#List_of_allowed_URL_characters) in the manner of "http:" URLs. Reserved characters in a SAFE URL have special meanings, but will be treated without that special meaning if they appear [percent encoded](https://en.wikipedia.org/wiki/Percent-encoding).

So where a URL includes a "?" that is **not** start of a *query_string*, it would need to be given as "%3F". Also, for "&" to appear in a SAFE URL it will need to be given as "%26" etc.

#### Plugin Behaviour

Immediately prior to the GET, (i.e. after scanning for "?" and stripping any query string), the plugin will scan each SAFE URL and replace any *percent encoded* sequences with their ASCII equivalents. This includes both the URL for the page to be loaded, and any content URLs obtained from the query string.

#### Further Consideration

1) For server side scripting there are established standards for this kind of API, such as the [Common Gateway Interface (CGI) specification]() for web servers. I'm not sure if there will be anything for us to borrow from this. It might help us think about whether any standard environment variables would be useful, such as for the SAFE URL of the main page, and how to name them.

2) It might be necessary or useful to allow a more compact way of specifying content URLs so that a full URL is not required to be specified in each case, but this has not been considered at this stage. If the RFC is accepted, we might consider whether the scheme needs to include a method for abbreviation in order to avoid very long URLs. This might be achieved by allowing a variable value to contain references to another query string variable, that is replaced with the variable value before issuing a GET.

3) For further speed enhancement, we might consider whether the query string could extend to including URLs of objects (e.g images, scripts) that might be embedded in the HTML of the page, so these can be pre-fetched to the browser cache even before the page itself has loaded.

# Drawbacks

This requires the plugin to support concurrent GET requests and provide additional API to the client, including support for client side processing of asynchronous GET responses and error conditions.

# Alternatives

None noted. I can't see another way of supporting dynamically generated HTML pages.

# Unresolved questions

Confirmation of the special characters, translation and any implementation issues arising from extending the plugin client side API need to be considered, plus consideration of any related features needed to assist web apps on SAFE, that have not been noted yet.
