# Linked Data WIP - ideas only

- Status: proposed 
- Type: new feature
- Related components: (safe_core, client, vault, routing (data))
- Start Date: 01-09-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this) 
- Supersedes:  None
- Superseded by: N/A

## Summary

Whereas appendable data as per [RFC38](https://github.com/maidsafe/rfcs/blob/master/text/0038-appendable-data/0038-appendable-data.md)may be considered an application defined one-way link, this RFC proposed fully bi-directional linked data.  One-way linked data allows the formation of a strongly connected data set (such as log, blog comments, messages etc.) bi-directional linked data promises to enhance data queries and move closer to semantic web type capabilities.

## Motivation

With the introduction of AppendableData there is an oportunity to create two way connections in "real time" between data sets. This linking capability can be exploited to allow graph analysis, permissioned sharing of data and more. In similar vein to [SOLID](https://solid.mit.edu/) approach this would put people in charge of their sharing habits, either by type or by application. Unlike similar systems the owner of the data in SAFE is anonymous. 

Types may be components of or complete: microblogs, commenting systems, medical records, software, videos, music, land registry information, mapping services and more.

## Detailed design

A seperate Id is used in this case and will require a new reserved type per data type to be used. This type will allow users to host/link to a multitude of data elements. As links between data types are in themsleves security leaks each data type should have it's own unique address. Using a SQRL type mechanism (basically seeding form known unique data) there is no requirement for a client to hold many keypairs. 


## Drawbacks

Why should we *not* do this?

## Alternatives

What other designs have been considered? What is the impact of not doing this?

## Unresolved questions

What parts of the design are still to be done?
