# Vault RPC

- Status: proposed
- Type: New Feature
- Related components: [safe_vault](https://github.com/maidsafe/safe_vault), [Vault Config File](https://github.com/maidsafe/rfcs/blob/master/text/0015-vault-config-file/0015-vault-config-file.md)
- Start Date: 05-07-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes:
- Superseded by:

## Summary

This RFC outlines a remote procedure call (RPC) module for interacting with vaults.

## Motivation

Monitoring and interacting with vaults is an important activity for vault operators.

Vault data is currently available via the log. However, extracting data from the vault requires parsing the log which is prone to error and often computationally expensive.

Making changes to the state of the vault requires restarting it.

Providing remote procedure calls for the vault would make inspection and changes much simpler and expand opportunity for additional tooling around vaults.

## Detailed design

The proposed design uses [HTTP JSON-RPC](http://json-rpc.org/wiki/specification) to communicate directly with the vault, and has been heavily influenced by the [Bitcoin RPC API](https://en.bitcoin.it/wiki/API_reference_%28JSON-RPC%29) and the [Bitcoin API Calls List](https://en.bitcoin.it/wiki/Original_Bitcoin_client/API_calls_list)

Disabling the RPC service should be possible, as should running on a port specified by the user.

Authentication must be required to execute RPC calls to prevent unauthorized changes.

Some new options should be added to the existing vault configuration file:

### config
```
{
  ...
  "rpcallowip": [
    "123.45.67.89",
    "98.76.54.32"
  ],
  "rpcenabled": true,
  "rpcport": 4001,
  "rpcuser": "some_username",
  "rpcpassword": "some_secure_password",
  ...
}
```

Proposed calls are outlined below. The parameter convention is &lt;name&gt; for *required* parameters and [name] for *optional* parameters.

### addnode &lt;address&gt; &lt;add/remove&gt;
```
adds or removes a node in the routing contacts list, using ip:port to specify the address
```

### getinfo
```
returns information about the configuration of the vault
{
  "max_capacity": 100000000,   // bytes
  "port": 4000,
  "rpc_version": "0.0.1",
  "vault_version": "0.10.2",
  "wallet_address": "245df3245df3245df3245df300000000000000001cc0dd1cc0dd1cc0dd1cc0dd"
}
```

### getpeerinfo
```
returns information about the current peers in the routing table
[
  {
    address": "14.56.34.194:4000",
    version": "0.10.2"
  },
  ...
]
```

### getstats
```
returns statistics about the operation of the vault
{
  "bandwidth_consumed": 300240723050,   // bytes
  "data_stored": {
    "id": 3,
    "sd": 5,
    "total": 2827592   // bytes
  },
  "disk_usage": {
    "size": 237820568,      // bytes
    "used": 173923180,      // bytes
    "availabe": 51793708    // bytes
  },
  "hops": {
    "ack": 1960,
    "connection_info": 6,
    "expect_close_node": 7,
    "group_message_hash": 133,
    "get_close_group": {
      "requests": 29,
      "responses": 52
    },
    "get_node_name": {
      "requests": 1,
      "responses": 2
    }
  },
  "load": {
    "avg_1m": 0.43,
    "avg_5m": 0.30,
    "avg_15m": 0.26,
    "newest_process_id": 22901,
    "running_processes": 1,
  },
  "messages": {
    "bytes": 16080019,
    "direct": {
      "connection_unneeded": 0,
      "new_node": 2,
      "node_identify": 5
    },
    "sent": 2500,
    "uncategorized": 2,
  },
  "requests": {
    "delete": {
      "request": 0,
      "success": 0,
      "failure": 0
    },
    "get": {
      "request": 41,
      "success": 39,
      "failure": 0
    },
    "post": {
      "request": 12,
      "success": 12,
      "failure": 0
    },
    "put": {
      "request": 39,
      "success": 39,
      "failure": 0
    }
  },
  "routing_table_size": 35,
  "system_time": 1467347765,   // unix epoch
  "system_uptime": 10410,      // seconds
  "system_uptime_idle": 5020,  // seconds
  "vault_uptime": 10300,       // seconds
  "vault_uptime_idle": 1000    // seconds
}
```

### help [command]
```
lists these commands or details for the specified command
```

### restart
```
restarts the vault
```

### setmaxcapacity &lt;capacity&gt;
```
use a new max_capacity value and save it to the configuration file
```

### setwalletaddress &lt;address&gt;
```
use a new wallet_address value and save it to the configuration file
```

### stop
```
stops the vault
```

## Drawbacks

RPC requires a strong password for authentication. There may be unintended changes to the vault if accessed maliciously.

RPC consumes additional system resources and a poorly configured client may be a source of denial of service attacks.

There is a need to balance the goals of RPC usage with the goals of logging, so functionality of both is retained but without excessive duplication.

## Alternatives

An appropriate alternative would be to create a standalone service that can parse vault logs and update the configuration. This may require changes to vault logging and adding a 'reload' option to the vault to reload new configurations without restarting.

## Unresolved questions

Should JSON-RPC be used, or an alternative such as REST, XML-RPC, or a simple http service?

What is an appropriate security model for this feature? Is username:password via http basic auth appropriate? This would require SSL to be secure when accessed from a remote machine, which is complex and possibly costly to set up. However, [bitcoin does not encourage use of SSL](https://en.bitcoin.it/wiki/Enabling_SSL_on_original_client_daemon) with the RPC client and instead suggests using SSH tunneling to provide the required security. Given this pattern of use, the configuration option for `rpcallowip` may be eliminated and only local access be allowed.

What information should be present in `getpeerinfo`? The [response from this call in bitcoin](http://chimera.labs.oreilly.com/books/1234000001802/ch06.html#on_a_node_runni) is quite extensive.

How much information should be contained in `getstats`? How should the data in `getstats` be structured? How should cross-platform differences be handled, such as platforms that have no information for system uptime or load averages?

What features may be added in the future? Is it simple to extend and modify the API? Should the API include version numbers in the response?

What errors may be encountered? How does the API respond to these errors?

Which RPC library should be used to implement this feature? The [list of JSON-RPC implementations](https://en.wikipedia.org/wiki/JSON-RPC#Implementations) does not currently have any entries for rust.
