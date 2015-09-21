- Feature Name: Prevent any single address being used more than once during address relocation.
- Type enhancement
- Related components routing, safe_vault
- Start Date: 21-09-2015
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Prevent a potential attack where a node gains overall/majority influence in a group setting.

# Motivation

The possibility 'may' exist for a node on the network to generate off-line derived identities close to a specific target and to then connect nodes whose resultant network relocated identities actually lie close to that target. In such cases the owner of that node will dominate, or at least have a dominant influence within, the group close to that target. This RFC proposes an amendment to the current process to prevent this possibility arising in practice.

# Detailed design

A node joining the SAFE network generates a new cryptographic key pair to access the network for each session. The generated public key, K, is then cryptographically hashed using some hash function H to give H(K). The node then sends a message, containing K, to the group H(K) requesting a network identifier for the session. On receipt of such a request each node belonging to the group hashes the concatenation of K with the two closest nodes, N1, N2 say, in xor distance from H(K) present in their routing tables, giving H(K + N1 + N2) = N, the identifier the node will use on the network for the session.

Having computed N, each node in the group H(K) sends a message, containing K, to the close group of N to confirm address relocation has been applied for the node owning K. A sufficient condition for the receiving group to resolve the message is for a quorum in the sending group to agree on the two closest nodes. Each node in the group H(K) further sends a message, containing N, to the joining node confirming the network address it now occupies.

In it's current form the process ensures that a joining node cannot occupy a position of it's own choosing. However, in a semi-stable network state it may be possible given the joining node knows the two closest nodes, N3, N4 say, in it’s joined group to generate keys that are relocated close to some identifier, I say, and thus obtain group influence at that location. It is the purpose of this RFC to avoid that possibility by storing the two close node identities, N3 and N4, at each node belonging to the group close to I in order to reject relocations involving either of those nodes if they occur more than once for the participating nodes in a given session.

# Drawbacks

An analysis of the complexity of producing network identifiers at will under various states of network churn and size could provide a contradictory perspective.

# Alternatives

The analysis in the Drawbacks section above provides motivation to render the current process sufficient in terms of network security.

# Unresolved questions

The intention is to implement the currently proposed design in the absence of evidence contradictory to it's requirement.
