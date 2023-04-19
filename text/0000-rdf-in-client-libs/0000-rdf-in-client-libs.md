# RDF support in the SAFE Client Libs

- Status: proposed
- Type: new feature
- Related components: SAFE Client Libs, SAFE Vault
- Start Date: 03-12-2018
- Discussion: http://
- Supersedes: n/a
- Superseded by: n/a

## Summary

This RFC outlines the features that we'll be adding as an integral part of SAFE Network on the application level. The questions of actual implementation strategies are out of scope of this RFC.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation

RDF ([Resource Description Framework](https://www.w3.org/RDF/)) is a serialisation-agnostic data representation framework first specified by the W3C organisation in 1998. Several complementary W3C recommendations, such as [RDF Schema](https://www.w3.org/TR/rdf-schema/), define the standard way of structuring data. Many widely-used conventional schemas already exist to represent things like social media user profiles or social connections. It is desirable to use these conventions and standards to structure data stored on the SAFE Network instead of relying on our own proprietary formats.

The RDF design also enables Linked Data [1], which is one of the main components of the [Semantic Web](https://www.w3.org/standards/semanticweb/). Considering the SAFE Network fundamental of storing data in perpetuity, it can provide significant benefits in this area as it alleviates the problem of link rot and data archival.

Furthermore, supporting RDF standards would also allow the SAFE Network to be compatible with [Solid](https://solid.mit.edu), the decentralised Web project created by Tim Berners-Lee and Inrupt. The compatiblity layer would allow to interoperate with applications created for Solid, and Vaults can serve the role of Solid _data pods_.

RDF should be seen as a higher-level framework to work with general-purpose data, sitting on top of network-level primitives such as Mutable Data, Immutable Data, or any other primitive data type existing on the SAFE Network.

While clients were already capable of using RDF on the SAFE Network, there was no uniform way of doing this. Adding these capabilities to the [SAFE Client Libs](https://github.com/maidsafe/safe_client_libs) would make it a part of our API intended to be used by application developers and would allow to use the same API across different platforms and languages. It would also allow to use RDF to represent core primitives and abstractions, such as NFS files or containers, in a standard-conforming way.

In addition to APIs for structuring and linking arbitrary data using RDF, SAFE Client Libs would provide an extra way to retrieve and manage data on the SAFE Network: a simple, SQL-like query language [SPARQL](https://www.w3.org/TR/rdf-sparql-query/). Having SPARQL in the Client Libs would allow to fetch data from multiple data sources with a single query (federated queries). This would effectively replace the need to use proprietary or complicated protocols of retrieving data, such as REST APIs on the clearnet or Mutable Data API on the SAFE Network. If application developers would prefer other graph query languages ([GraphQL](https://graphql.org/), [LDlex](https://github.com/RubenVerborgh/LDflex), etc.), it is possible to translate them into SPARQL.

## Detailed design

### RDF triples storage

RDF resources and triples are serialised and stored on the SAFE Network using one of the existing data types. It MUST be agnostic to which particular data type is used.

### RDF and immutability

Data immutability is one of the [SAFE Network fundamentals](https://safenetwork.tech/fundamentals/) and it MUST be maintained for RDF triples too.

With the evolving nature of RDF resources, we need to make sure we support the ability to change _and_ archive the resources [2]. This network fundamental also has a great advantage for storing _linked data_ because links will not rot (as it might happen on the clearnet).

### Triple storage serialisation format

There are multiple requirements for the triple storage serialisation format. It MUST support:

* Encryption (allowing to store public or private resources on the network)
* Querying (on both public and private data)
* Data archival (all changes MUST be retained)
* Upgrades (retaining compatibility with previously used versions of the serialisation format once it's changed)

These requirements are non-exclusive; i.e., it MUST be possible to query an older version of an RDF resource which is also encrypted.

The actual serialisation format and storage characteristics are not defined by this RFC and are left to the developer's discretion. A developer MAY choose to support multiple serialisation formats due to the backwards compatiblity requirements or efficiency reasons (e.g., in this case the upgrades requirement can be satisfied by supporting older serialisation formats).

### Querying capabilities

The provided API MUST support a way to query triples data using the [SPARQL 1.1 query language](https://www.w3.org/TR/sparql11-query/). `safe_app` must provide functions that can be used to execute SPARQL queries on the RDF resources stored on the network.

It MUST also support [_federated_ queries](https://www.w3.org/TR/sparql11-federated-query/), querying from multiple RDF resources stored at different locations on the Network.

The query processor MUST support using the [XOR-name based URLs](https://github.com/maidsafe/rfcs/blob/master/text/0053-xor-urls/0053-xor-urls.md) and the SAFE [Public Name System](https://github.com/maidsafe/rfcs/blob/master/text/0052-RDF-for-public-name-resolution/0052-RDF-for-public-name-resolution.md) for specifying the data source locations.

Vaults MAY support queries on data they store, but this might require having a query protocol, which is out of scope of this RFC.

### SAFE Client Libs

We provide low-level API functions to work with RDF as a data model. The functions are provided as a part of the SAFE Core library and SHOULD NOT be feature-gated.

Rationale for adding this feature to our library stack (as opposed to having it on the application-level only) is to make it usable from all languages we support and to make RDF a part of the Client Libs core APIs. It is RECOMMENDED to provide higher level wrappers in the language bindings to simplify the RDF concepts for app developers.

We specify a set of functions that SHOULD be present in the SAFE Client Libs, but the actual API is left to be defined as an implementation detail.

SAFE Client Libs MUST provide all RDF and querying capabilities irrespective of Mock or Non-Mock mode of operation.

SAFE Client Libs MUST provide an equivalent foreign funcation interface (FFI) to make the RDF API usable from other languages. It MUST conform to the [FFI calling convention](https://github.com/maidsafe/safe_client_libs/wiki/FFI-calling-conventions) used in the library. It SHOULD provide equivalent functions _and_ structures; however, due to the lack of type safety in FFI, types `Subject`, `Predicate` and `Object` MAY be collated into a single type `Node` to reduce the number of entities and simplify the API.

SAFE App MUST export the following public structures and enums representing the RDF model and utilising the static Rust type system to define elements of an RDF graph. It is RECOMMENDED to reuse these structures from an external Rust RDF library instead of defining our own.

```rust
/// IRI - internationalised URI.
pub struct Uri { .. }

/// Represents a literal (string, number, etc.)
/// stored in an RDF graph.
pub enum Literal {
    String {
        value: String,
        language: Option<String>
    },
    Typed {
        value: String,
        language: Option<String>,
        data_type: Option<Uri>,
    }
}

impl Literal {
    /// Return the string value of the `Literal`.
    fn value(&self) -> Option<&str>;
    /// Return the language of the `Literal`.
    fn language(&self) -> Option<&str>;
    /// Set a new value for the `Literal`.
    fn set_value(&mut self, new_val: String);
    /// Set a new language for the `Literal`.
    fn set_language(&mut self, lang: Option<String>);
}

/// Helper trait to work with subject/predicate/object in a uniform way.
trait Node {
    /// Return `true` if this node is blank
    fn is_blank_node(&self) -> bool;
    /// Return literal value of the node.
    /// If it is not a literal, return `None`.
    fn literal(&self) -> Option<&Literal>;
    /// Return URI value of the node.
    /// If it is not a node, return `None`.
    fn uri(&self) -> Option<&Uri>;
}

pub enum Subject {
    Uri(Uri),
    BlankNode
}

pub type Predicate = Uri;

pub enum Object {
    Uri(Uri),
    BlankNode,
    Literal(Literal)
}

impl Node for Subject { ... }
impl Node for Predicate { ... }
impl Node for Object { ... }

pub struct Triple {
    subject: Subject,
    predicate: Predicate,
    object: Object,
}

/// Represents an abstract RDF graph.
pub struct RdfGraph { .. }
```

Each of the following functions MUST be available as a part of a public API. It is RECOMMENDED to follow the provided API design.

```rust
/// It is assumed that the SAFE Core's `Client` trait is used
/// to perform the network operations.
use safe_core::Client;

impl RdfGraph {
    /// Create an empty RDF graph.
    pub fn new() -> RdfGraph;

    /// Add a new triple to the graph
    pub fn add_triple(&mut self, triple: Triple);

    /// Remove a triple from the graph
    /// Returns the removed triple.
    pub fn remove_triple(&mut self, triple: Triple) -> Triple;

    /// Returns true if the graph contains the provided triple
    pub fn has_triple(&self, triple: &Triple);

    /// Returns a mutable reference to a triple if it's contained
    /// in the graph
    pub fn get_mut(&mut self, triple: &Triple) -> Option<&mut Triple>;

    /// Returns the size of the graph
    pub fn len(&self) -> usize;

    /// Sequentially iterate over triples contained in this RDF graph.
    pub fn iter<'a>(&self) -> impl Iterator<Item = &'a Triple>;

    /// Sequentially iterate over triples contained in this RDF graph
    /// with an option to mutate them.
    pub fn iter_mut<'a>(&mut self) -> impl Iterator<Item = &'a mut Triple>;

    /// Save an RDF graph to a specified location on the network.
    /// This function MAY replace `XorName` with another data
    /// pointer type. (e.g. MDataInfo).
    /// The arguments MAY require providing an encryption key
    /// if the data should be private.
    /// This function will create a new resource on the network and
    /// store triples using a preferred data type.
    /// If an RDF resource is already stored at the specified location,
    /// a new graph version should be created or existing data should
    /// be edited in place (depending on the semantics of the data
    /// type being used)
    pub fn store(&self, client: impl Client, name: XorName)
        -> impl Future<Item = (), Error = RdfError>;

    /// Load an RDF graph from a specified location on the network.
    /// This function MAY replace `XorName` with another data
    /// pointer type. (e.g. MDataInfo).
    /// The arguments MAY require providing an encryption key if the
    /// data to be fetched is private.
    /// To make the API future-proof, it is RECOMMENDED to have arguments
    /// specifying a particular version of an RDF resource to be fetched.
    pub fn fetch(
        client: impl Client,
        name: XorName,
        version: Option<u64>
    ) -> impl Future<Item = Self, Error = RdfError>;

    /// Run a SPARQL query locally on the existing RDF graph.
    /// Returns a new RDF graph with the query results.
    pub fn run_sparql_query(&mut self, query: &str)
        -> impl Future<Item = RdfGraph, Error = QueryError>;
}

/// Run a SPARQL query on a remote data source.
/// The `uri` contains a XOR URI (as defined in the RFC 53 [1]),
/// pointing to a resource on the SAFE Network. It could also point to
/// a public address [2]. It MUST NOT support any other protocols and
/// URI schemas such as `http://` or `https://`.
///
/// If the URI is not specified, it is assumed that a URI is provided
/// as a part of the query string.
///
/// The query SHOULD be processed asynchronously.
///
/// [1]: https://github.com/maidsafe/rfcs/blob/master/text/0053-xor-urls/0053-xor-urls.md
/// [2]: https://github.com/maidsafe/rfcs/blob/master/text/0052-RDF-for-public-name-resolution/0052-RDF-for-public-name-resolution.md
pub fn run_sparql_query(
    client: impl Client,
    uri: Option<&str>,
    query: &str
) -> impl Future<Item = RdfGraph, Error = QueryError>;
```

#### Serialisation

It is RECOMMENDED to implement the serialisation formats as a trait:

```rust
trait RdfSerialisation {
  fn serialise(model: &RdfGraph) -> String;
  fn deserialise(source: &str) -> Result<RdfGraph, RdfError>
}
```

We do not depend on any particular serialisation format, but it is RECOMMENDED to support the [RDF 1.1 Turtle](https://www.w3.org/TR/turtle/) and [JSON-LD](https://json-ld.org/) serialisation formats by default.

## Drawbacks

* Developers might be not familiar with RDF concepts which sometimes are counter-intuitive and challenging. However, this can (and should) be solved at the application level by supporting user-friendly RDF schema or SPARQL wrappers and languages such as [LDFlex](https://github.com/solid/query-ldflex/) or formats like [JSON-LD](https://json-ld.org/), making the Linked Data usage transparent for a developer.

* The new features expand the scope of SAFE Client Libs further, adding a component that is not related to the SAFE Network _per se_.

## Alternatives

### Putting RDF in a separate library

Instead of adding this as a new feature to SAFE Core, we can consider having RDF as a separate library. However, considering we're aiming to move many of the core data structures (such as NFS containers and name services) to use the conventional schemas, it would make more sense to have RDF as a part of the SAFE Client Libs core.

### Supporting RDF in the Vaults

One of the alternatives is to support RDF natively by the Vaults, either in form of a separate data type or as an extra layer on top of existing data types.

The major advantage that this approach can bring is the native support for complex queries on RDF triples, which in turn would allow to support high-level query languages such as SPARQL on the Vaults level, maximising the queries efficiency.

While there might be many other benefits, there are also a lot of implementation challenges that can overcomplicate the Vaults code and require a significant implementation effort. To name a few:

* For complex queries, how do we limit the resource usage on the Vaults side?
* If it is an extra layer on top of an existing data type, how do we maintain a uniform triple serialisation format across clients and vaults? What if it will need to be updated?
* It makes Vaults and the network _smart_ by adding awareness of the higher-level concepts and data types (as e.g. if we added NFS support to Vaults). This breaks the separation of concerns principle. Arguably, Vaults should be kept as simple as possible, dealing with data only at a very primitive level (similarly to block storages).

## Unresolved questions

- Queries efficiency: as the queries are executed on the client side only, the clients might need to fetch the RDF resources in their entirety to execute a query. This might be inefficient for large resources and complex queries. An efficient triple serialisation format or a query protocol (similar to HTTP/1.1-based [protocol used in SPARQL](https://www.w3.org/TR/sparql11-protocol/)) might be required to alleviate this, but this topic is not in the scope of this RFC.

## References

[1] "Linked Data - The Story So Far", Bizer, Heath, Berners-Lee, http://tomheath.com/papers/bizer-heath-berners-lee-ijswis-linked-data.pdf

[2] "Towards Efficient Archiving of Dynamic Linked Open Data", Fern ́andez et al., http://ceur-ws.org/Vol-1377/paper6.pdf
