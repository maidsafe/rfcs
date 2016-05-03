- Feature Name: Add messaging example to `safe_core`
- Status: proposed
- Type: Enhancement
- Related components: [safe_core](https://github.com/maidsafe/safe_core)
- Start Date: 09-02-2016
- RFC PR: (leave this empty)
- Issue number: (leave this empty)

# Summary

Expand the current example in `safe_core` to include the features of creating an mpid_account for messaging and send/receive a message to/from a peer user.

# Motivation

The current `safe_core` has an example `safe_client` showing how a user can create an account and login to the Vault network. With the messaging feature now having been implemented on the Vault and Client side, we wish to expand the `safe_core` example to include this messaging feature. This can shall showcase how secured messaging works within the distributed Vault network, and also to testify the current implementation from both Vault and the Client side.

# Detailed design

The account creation and login part within the current `safe_client` will remain unchanged.

Once a client logged in, an option of creating an mpid_account will be provided.
The user is allowed to use any name, say Alice, as a memorable account name.
```rust
/// safe_client code for create mpid_account
let _ = create_mpid_account(client, "Alice");

fn create_mpid_account(client: &Client, account_name: &String) -> Result<ResponseGetter, CoreError> {
    let mpid_account = XorName(hash512(account_name).0);
    client.register_online(mpid_account);
}
```

After that, a sequence of CLI interactions will ask the user to input the receiver's memorable name, metadata and content of the message.
```rust
/// safe_client code for sending an mpid message
fn send_mpid_message(client: &Client, mpid_account: &XorName) {
    let _ = std::io::stdin().read_line(&mut receiver_name);
    let receiver_account = XorName(hash512(receiver_name).0);
    let _ = std::io::stdin().read_line(&mut msg_metadata);
    let _ = std::io::stdin().read_line(&mut msg_content);
    let secret_key = client.account.unwrap().get_maid().secret_keys().0.clone();
    let _ = client.send_message(mpid_account, msg_metadata, msg_content, receiver_account, secret_key);
}
```

The second `safe_client` can now be started up and once Client is logged in, the previous used receiver's name shall now be used to create an mpid_account for this Client. It is then expected that the previously sent message should now be received by this client.
```rust
/// safe_client code for receiving an mpid message
let response_getter = try!(create_mpid_account(client, "Bob"));
loop {
    match response_getter.get() {
    	Ok(data) => {
    		match data {
    			DataRequest::PlainData(plain_data) => {
                    let mpid_message_wrapper : MpidMessageWrapper = try!(deserialise(plain_data));
                    println!("received mpid message {:?}", mpid_message_wrapper);
                    break;
    		    }
    	    }
    	}
    	Err(_) => {}
    }
	sleep_ms(1000);
}
```


# Drawbacks

1, To avoid exchanging long hex code mpid_account, the example code needs to carry out a hash function, so that it is a human readable memorable word that can be used as an mpid_account name.

2, Two `safe_client`'s need to be executed in sequence to demo the messaging feature.

3, The secret key from maid_account has been used as the signing secret key for the mpid messaging.

# Alternatives

N/A

# Unresolved questions

N/A
