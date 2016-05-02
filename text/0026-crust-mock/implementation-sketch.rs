
// Note: lifetime and borrowing issues are glossed over here to keep the high-level
// logic and structure easier to reason about.

mod mock {

  // Mock network used to facilitate routing of network messages between mock
  // peers.
  //
  // This might be optionally used to intercept and examine the messages,
  // although that might be better done on the peers.
  struct Network {
    peers: HashMap<Endpoint, &Peer>,
    queue: VecDeque<(Endpoint, Endpoint, Event)>

    // TODO: some kind of condition variable here. Need to look up how to use those.
  };

  impl Network {
    fn new() -> Self {
      //...
    }

    fn send(&mut self, sender: Endpoint,
                       receiver: Endpoint,
                       event: Event)
    {
      self.queue.push_front((sender, receiver, event));
      // TODO: notify the cond var here
    }

    fn wait_and_process_events(&mut self) {
      wait_for_events();
      process_events();
    }

    // Call this to process all accumulated events. This can be used to control
    // when if the network event are delivered.
    fn process_events(&mut self) {
      while let Some((sender, receiver, event)) = self.queue.pop_back() {
        let peer = self.peers.get(receiver).unwrap();
        peer.receive_from(sender, event);
      }
    }

    fn wait_for_events(&self) {
      // TODO: wait on the cond var here
    }
  }

  // Network event. This abstract over various network operations like TCP
  // connect/accept, UDP rendezvous connect, sending/receiving network packets,
  // etc...
  enum Event {
    // Attempt to connect
    // TODO: we might need TcpConnect/TcpAccept, UdpConnect/UdpAccept,
    // RendezvousConnect, ...
    Connect,
    ConnectSuccess,
    ConnectFailure,

    // Disconnect
    Disconnect,

    // Send message (packet)
    Send(Vec<u8>),
  }

  // Mock peer. It simulates a small subset of a real physical machine connected
  // to a network (mostly the config files and network sockets).
  struct Peer {
    network: &Network,
    endpoint: Endpoint,
    service: Option<&Service>,
  };

  impl Peer {
    fn new(network: &Network, endpoint: Endpoint, config: Config) -> Self {
      let peer = Peer {
        network: network,
        endpoint: endpoint,
        service: None,
      }

      network.peers.insert(endpoint, peer);
      peer
    }

    fn send_to(&self, receiver: Endpoint, event: Event) {
      self.network.send(self.endpoint, receiver, event);
    }

    fn receive_from(&self, sender: Endpoint, event: Event) {
      if !self.filter_event(&sender, &event) {
        return;
      }

      self.process_event(sender, event);
    }

    fn filter_event(&self, sender: Endpoint, event: &Event) -> bool {
      // Here we can decide whether to accept the event or reject it, based on
      // various criteria. This can then be used for example to simulate
      // different types of NATs.
    }

    fn process_event(&self, sender: Endpoint, event: Event) {
      match event {
        // For example
        Connect => {
          if self.listening_tcp {
            self.send_to(sender, Event::ConnectSuccess);
            self.service.unwrap().on_accept(sender);
          } else {
            self.send_to(sender, Event::ConnectFailure);
          }
        }

        Send(content) => {
          self.service.unwrap().on_receive(sender, content);
        }

        // and so on...
      }
    }
  }

  // Mock `crust::Service`
  struct Service {
    peer: &Peer,
    connections: Vec<(Endpoint, PeerId)>
  };

  impl Service {
    fn new(peer: &Peer, event_tx: CrustEventSender, service_discovery_port: u16)
      -> Result<Service, Error>
    {
      // ...
      peer.service = Some(&self);
      // ...
    }

    // Other methods like in `crust::Service`, for example:
    fn start_listening_tcp(&mut self) {
      self.peer.listening_tcp = true;
    }

    fn send(&self, peer_id: &PeerId, data: Vec<u8>) -> io::Result<()> {
      // TODO: error handling
      let endpoint = self.peer_id_to_endpoint(peer_id);
      self.peer.send_to(endpoint, Event::Send(data));
    }

    // ...and so on...

    // Additional methods used for example by Peer to notify this Service about
    // events. Not part of the original crust::Service interface.
    fn on_connect(&self, peer_endpoint: Endpoint);
    fn on_accept(&self, peer_endpoint: Endpoint);
    fn on_receive(&self, peer_endpoint: Endpoint, content: Vec<u8>)
    // etc...
  }

}

// We are going to need additional way to construct `routing::Core`s (could be
// behind the same feature gate as the rest of the mocking stuff.)
impl Core {
  fn new_with_peer(peer: &Peer,
                   // the rest of the arguments are the same as `Core::new`.
                   event_sender: Sender<Event>,
                   client_restriction: bool,
                   keys: Option<FullId>)
    -> Result<(RoutingActionSender, RaiiThreadJoiner), RoutingError>
    {
      // ...
      let service = mock::Service::new(peer, event_tx, service_discovery_port);
      // ...
    )
}


// Usage
#[test]
fn test_mock_service() {
  let network = mock::Network::new();
  let peer0 = mock::Peer::new(network, random_endpoint(), Config::default());
  let peer1 = mock::Peer::new(network, random_endpoint(), Config::default());

  let (event_tx0, event_rx0) = mpsc::channel();
  let (action_sender0, _joiner0) = routing::Core::new_with_peer(&peer0,
                                                                event_tx0,
                                                                true,
                                                                None).unwrap();

  let (event_tx1, event_rx1) = mpsc::channel();
  let (action_sender1, _joiner1) = routing::Core::new_with_peer(&peer1,
                                                                event_tx1,
                                                                false,
                                                                None).unwrap();

  network.wait_and_process_events();

  match event_tx0.recv() {
    Ok(routing::Event::Connected) => {
      // TODO: Test stuff...
    }
  }
}
