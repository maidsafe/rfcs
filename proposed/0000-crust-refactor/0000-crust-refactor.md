- Feature Name: crust_redesign
- Type: enhancement
- Related components: crust
- Start Date: 02-11-2015
- RFC PR:
- Issue number:

# Summary

The current design of crust is inefficient and cumbersome to use. This RFC
proposes a restructure of crust and it's API.

# Motivation

* Efficiency

The current design has major efficiency problems. It is extremely thread-heavy.
For every connection there are 3 threads running to manage the connection. Even
performing small actions involves lot of unnecessary allocation, copying and
context switching. For example, to send data to a peer a closure and an owned
buffer must be allocated and the data copied into it. That buffer is then
bounced between 4 threads: the originating thread, the `State` thread, the
writer thread and the transport thread before finally arriving at where it gets
sent down the wire. Reading from multiple connections is implemented by having
multiple threads each reading from a single socket and all sending their data
down a channel to the `State` thread. Using select/epoll for this would be much
more scalable. The `State` thread is a point of contention as it is a single
crust-wide thread that all IO events must pass through, killing the performance
benefits of multi-threading.

* Usability

In the current design, all events that crust generates are sent to a single
channel. This forces the user to have a single thread reading and dispatching on
these events. One effect of this is that it prevents the user from writing
truly parallel code. Another is that it prevents the user from being able to
write code with a logical control structure. For example, to connect to a peer
it would be nice if the user could write something like

    let connection = service.connect(address);

But instead they are forced to initiate the connection and process the result
separately, using tokens to figure out which results correspond to which
actions they initiated. Having a central point in their code processing all
crust events prevents the user from being able to completely decouple independent
parts of their code that may want to use crust independently. Consider UDP hole
punching; this happens in two stages: the user requests a mapped socket and
then uses the socket to punch a hole. Currently, the user of crust acheives
this as follows:

  * One part of the code requests a mapped socket with a result token.
  * A completely separate part of the code keeps a lookout for the result
    event with the corresponding token while simultaneously handling all
    other unrelated events such as packets arriving or new peers connecting.
  * When the result event is seen it is dispatched to a part of the code that
    initiates the hole punching.
  * The receiver/dispatcher thread again looks for the corresponding result.

Instead, it might be nice if the user could implement the hole punching
procedure in a function:

```rust
// psuedo-code
fn do_hole_punching() -> (UdpSocket, SocketAddr) {
    let socket = get_mapped_socket();
    udp_punch_hole(socket)
}
```

Even better, it would be nice if this procedure could be implemented inside
crust itself. This currently isn't possible without the ability for crust to
monitor and filter it's own events.

* Other issues

  * Error handling. Crust is currently lacking in it's error handling plumbing.
    For example, errors that occur in the `State` thread have nowhere sensible
    to be sent and are currently just discarded.
  * Resource leakage. Crust likes to spawn threads everywhere for every action.
    This makes it easy to write code where a thread can block forever and never
    get cleaned up.

# Detailed design

## Design goals

Everything considered, what sort of design goals should crust strive for? In my view, some
sensible goals are to be:

* Close to the metal

  Crust should create a minimal amount of overhead as possible without
  sacrificing practicality. This means, for example, sending/receiving on tcp
  sockets should translate almost directly to write/read system calls without
  intermediate allocation or thread-hopping. Reading from multiple connections
  should happen via epoll/select without spawning any unnecessary threads. For
  this, we can use the poll implementation found in `mio`.

* Blocking-based

  Crust's API should be based on blocking calls like the rust standard library
  APIs. This would allow the implementation of functions such as
  `do_hole_punching` from the previous section. Note that blocking
  calls are strictly more flexible than other paradigms such as callbacks. This
  is because any blocking function can be made into a callback-based function
  simply by executing it another thread with a callback. ie. it's possible to
  write the following function:
  
  ```rust
  fn callbackerize<A, R, B, C>(blocking: B, arg: A, callback: C)
      where B: FnOnce(A) -> R,
            C: FnOnce(R)
  {
      thread::spawn(move || callback(blocking(arg)));
  }
  ```
  
  A reason blocking calls are not more popular in other languages is due to the
  difficulty in writing safe, truly concurrent code in other languages. Often,
  programs in these languages are based on a single thread executing an event
  loop where any time-consuming process must be handled asyncronously in order
  to allow execution to return to the event loop. This is a hard requirement in
  runtimes such as NodeJS, which don't have true multithreading, and is a
  common pattern in languages like C(++) where highly-multithreaded programming
  is dangerous. Rust's lifetime and borrowing semantics solve the problems
  associated with multithreaded programming in languages like C. In rust, it's
  possible to create multiple threads and freely share data between them except
  with the guarantees that threads cannot clobber each other by reading/writing
  data at the same time and that data structures will never be destroyed while
  they are still in use. The ability to write blocking, concurrent code is a
  major selling point of these features and something that crust should take
  advantage of.

* Safe

  Crust should leverage rust's type system and libraries to make it difficult
  or impossible to write broken code. For example, on destruction of a crust
  `Service` the user should be able to statically guarantee that all resources
  associated with the service have been cleaned up.

## Components

The unix philosophy is that programs (or libraries as it may be) should do one
thing and do it well but crust currently performs several distinct tasks. Some
of these tasks are

* Local peer discovery
* Nat traversal
* Transport abstraction
* Connection management

This RFC proposes that local peer discovery, NAT traversal and transport
abstraction be factored out into three separate libraries with Crust taking the
role of a connection management library that utilises the other three. The
proposed APIs of these libraries are introduced in four separate documents

* [beacon](beacon-library.md)
* [nat_traversal](nat-traversal-library.md)
* [transport](transport-library.md)
* [crust](crust-library.md)

## API style

I will now try to justify the design patterns used in these libraries.

The first thing to notice is how blocking operations are performed. In the rust
standard library, to perform a tcp connection one can write:

```rust
let stream = try!(TcpStream::connect(addr));
```

This is good, it creates a connection and returns it (or an error) to the place
in code where it's needed. However it has a problem, if the `connect` call
blocks for a long period of time there is no way to cancel it. This is
important as the part of the application making the connection may want to
cancel the connection attempt after some time period, or if another event
happens first, or if the application is shutting down, or any number of other
reasons. To enable this we can instead make the connection happen in two
stages. First we create a pair of objects, one of which is used to perform the
blocking call, the other of which is used to cancel the blocking call.

```rust
let (connect_go, connect_kill) = Stream::connect(addr);
```

We forward `connect_kill` to the part of the program that will be in charge of
cancelling the call and then perform the call.

```rust
sender.send(connect_kill)
let stream = connect_go.go();
```

Alternatively, `connect_go` could be sent to another thread to perform the
connection, the important part is that the `go` and `kill` are in two seperate
places of control. To cancel the operation we simply drop `connect_kill`.

As this is a common pattern throughout the libraries, there's a trait for it.

```rust
trait Go {
    type Output;

    fn go(self) -> Option<Self::Output> { .. }
    fn go_timeout(self, timeout: Duration) -> Option<Self::Output> { .. }
    fn go_deadline(self, deadline: SteadyTime) -> Option<Self::Output> { .. }
    fn go_opt_timeout(self, opt_timeout: Option<Duration>) -> Option<Self::Output> { .. }
    fn go_opt_deadline(self, opt_deadline: Option<SteadyTime>) -> Option<Self::Output>
}
```

Wherever types named `*Go` and `*Kill` occur in the libraries, this is the
pattern that's being used. Additionally, it's sometimes necessary to control a
blocking operation while it is taking place. In these cases, a `*Go` and
`*Controller` type are used where the controller is used to control the object
performing the operation. As with `*Kill`, dropping `*Controller` unblocks the
operation. For an example of this see the `Beacon::recv` method.

For operations that can be performed repeatedly the `*Go` type will return
itself upon success. For example, the `Output` of `ReadGo` is `(ReadGo, usize)`

To see how functions that use this blocking style can be used and composed
consider an example where a user wants to write a method which

 (0) Connects to an endpoint.
 (1) Writes "ping" to the stream.
 (2) Reads data from the stream.
 (3) Prints the result.

First, let's consider a simple example where we want to do these things in
sequence and timeout the whole operation after 1 second;

```rust
pub fn ping(endpoint: Endpoint) {
    let deadline = SteadyTime::now() + Duration::from_secs(1);

    let (connect_go, _k) = Stream::connect(endpoint);
    let stream = connect_go.go_deadline(deadline);

    let (write_go, _k) = stream.write(b"ping");
    let _ = write_go.go_deadline(deadline);

    let mut data = [0u8; 256];
    let (read_go, _k) = stream.read(&mut data[..]);
    let n = read_go.go_deadline(deadline);

    println!("data == {:?}", data);
}
```

Simple, right? For a richer example, consider how we implement this procedure
in a function which itself follows to go/kill style. This implementation allows
the ping operation to be killed gracefully if, say, the peer never responds.

```rust
pub fn ping(endpoint: Endpoint) -> (PingGo, PingKill) {
    let (connect_tx, connect_rx) = channel();
    let (write_tx, write_rx) = channel();
    let (read_tx, read_rx) = channel();
    let go = PingGo {
        endpoint: endpoint,
        connect_kill: connect_tx,
        write_kill: write_tx,
        read_kill: read_tx,
    };
    let kill = PingKill {
        _connect_kill: connect_rx,
        _write_kill: write_rx,
        _read_kill: read_rx,
    };
    (go, kill)
}

struct PingGo {
    endpoint: Endpoint,
    connect_kill: Sender<ConnectKill>,
    write_kill: Sender<WriteKill>,
    read_kill: Sender<ReadKill>,
}

struct PingKill {
    _connect_kill: Receiver<ConnectKill>,
    _write_kill: Receiver<WriteKill>,
    _read_kill: Receiver<ReadKill>,
}

impl Go for PingGo {
    type Output = ();

    fn go_opt_deadline(self, deadline: Option<SteadyTime>) {
        let stream = try_opt!(match Stream::connect(self.endpoint) {
            (go, kill) => self.connect_kill.send(kill).and_then(go.go_opt_deadline(deadline)) {
        });

        let _ = try_opt!(match stream.write(b"ping") {
            (go, kill) => self.write_kill.send(kill).and_then(go.go_opt_deadline(deadline)) {
        });

        let mut data = [0u8; 256];
        let n = try_opt!(match stream.read(&mut data[..]) {
            (go, kill) => self.read_kill.send(kill).and_then(go.go_opt_deadline(deadline)) {
        });

        println!("data: {:?}", data[..n]);
        Some(())
    }
}
```

## Mio usage

The APIs described in the other documents are intended to be efficiently
implementable and usable. For example, reading from a socket repeatedly should
not require us to spawn threads or repeatedly allocate data.

This can be acheived by building the `Acceptor`, `ReaderSet` and `WriterSet`
types around a `mio::Poll`. A `mio::Poll` can have many file descriptors
registered with it and then be used to block until one of them becomes active.

An example implementation of `Acceptor` is shown below:

```rust
pub struct Acceptor {
    inner: Mutex<AcceptorInner>,
    inner_freshened: Condvar,
    notify_write: Mutex<mio::unix::PipeWriter>,
}

struct AcceptorInner {
    poll: mio::Poll,
    listeners: Vec<TcpListener>,
    notify_read: mio::unix::PipeReader,
    fresh: bool,
    in_accept: bool,
    wake_buffer: u64,
}

impl Acceptor {
    pub fn new() -> Result<Acceptor, AcceptorNewError> {
        let (r, w) = try!(mio::unix::pipe().map_err(|e| AcceptorNewError::CreatePipe {cause: e}));
        let mut poll = try!(mio::Poll::new().map_err(|e| AcceptorNewError::CreatePoll {cause: e}));
        try!(poll.register(&r, ::TOKEN_NOTIFY, mio::EventSet::readable(), mio::PollOpt::level())
                 .map_err(|e| AcceptorNewError::RegisterNotify {cause: e}));
        Ok(Acceptor {
            inner: Mutex::new(AcceptorInner {
                poll: poll,
                listeners: Vec::new(),
                notify_read: r,
                fresh: false,
                in_accept: false,
                wake_buffer: 0,
            }),
            inner_freshened: Condvar::new(),
            notify_write: Mutex::new(w),
        })
    }

    pub fn accept<'a>(&'a mut self) -> (AcceptGo<'a>, AcceptorController) {
        let go = AcceptGo {
            acceptor: self,
        };
        let controller = AcceptorController {
            acceptor: self,
        };
        (go, controller)
    }
}
```

The `Acceptor` maintains a vector of `TcpListener`s, each of which is
registered with the `poll`. Additionally, there is a pipe (`notify_read`,
`notify_write`) which is registered for readability with `poll`. When
`AcceptGo` tries to accept on the socket set it aquires the `inner` `Mutex` and
blocks on `poll`. When `AcceptorController` wants to destroy or reconfigure the
`Acceptor` it notifies `AcceptGo` via the pipe, causing it to either return
`None` or temporarily release the `Mutex` so that `inner` can be mutated. This
is implemented below:

```rust
pub struct AcceptGo<'a> {
    acceptor: &'a Acceptor,
}

impl<'a> Go for AcceptGo<'a> {
    type Output = (AcceptGo<'a>, Result<Stream, AcceptError>);

    fn go_timeout(self, timeout: Duration) -> Option<(AcceptGo<'a>, Result<Stream, AcceptError>)> {
        let timeout_ms = timeout.num_milliseconds();
        let timeout_ms = match timeout_ms < 0 {
            true    => 0,
            false   => timeout_ms as usize,
        };
        loop {
            let mut inner = self.acceptor.inner.lock().unwrap();
            let n = match inner.poll.poll(timeout_ms) {
                Ok(n) => n,
                Err(e) => return Some((self, Err(AcceptError::Poll {cause: e}))),
            };
            for i in 0..n {
                match inner.poll.event(i).token {
                    ::TOKEN_NOTIFY => {
                        let mut c = [0u8];
                        read_exact(&mut inner.notify_read, &mut c[..]).unwrap();
                        match c[0] {
                            ::NOTIFY_WAKE => {
                                if inner.wake_buffer > 0 {
                                    inner.wake_buffer -= 1;
                                }
                                else {
                                    inner.fresh = false;
                                    while !inner.fresh {
                                        inner.in_accept = true;
                                        inner = self.acceptor.inner_freshened.wait(inner).unwrap();
                                        inner.in_accept = false;
                                    }
                                }
                            },
                            ::NOTIFY_SHUTDOWN => return None,
                            x => panic!("Unexpected byte in notify pipe: {}", x),
                        }
                    },
                    t => {
                        match inner.listeners[t.0].accept() {
                            Ok(None)         => (),
                            Ok(Some(stream)) => return Some((self, Ok(Stream::Tcp(stream)))),
                            Err(e)           => return Some((self, Err(AcceptError::Accept {cause: e}))),
                        }
                    },
                }
            }
        }
    }

    fn go_opt_timeout(self, opt_timeout: Option<Duration>) -> Option<(AcceptGo<'a>, Result<Stream, AcceptError>)> {
        self.go_timeout(opt_timeout.unwrap_or(Duration::max_value()))
    }

    fn go_opt_deadline(self, opt_deadline: Option<SteadyTime>) -> Option<(AcceptGo<'a>, Result<Stream, AcceptError>)> {
        self.go_timeout(opt_deadline.map(|d| d - SteadyTime::now()).unwrap_or(Duration::max_value()))
    }
}

pub struct AcceptorController<'a> {
    acceptor: &'a Acceptor,
}

impl<'a> AcceptorController<'a> {
    pub fn add_listener<A: ToListenEndpoint>(&self, addr: A) -> Result<(), AddListenerError<A::Err>> {
        let addr = try!(addr.to_listen_endpoint()
                             .map_err(|e| AddListenerError::ParseAddr {cause: e}));
        match addr {
            ListenEndpoint::Tcp(sa) => {
                let listener = try!(TcpListener::bind(&sa)
                                                .map_err(|e| AddListenerError::Bind {cause: e}));
                {
                    let mut notify_write = self.acceptor.notify_write.lock().unwrap();
                    try!(notify_write.write_all(&[::NOTIFY_WAKE])
                                     .map_err(|e| AddListenerError::Notify {cause: e}));
                }
                {
                    let mut inner = self.acceptor.inner.lock().unwrap();
                    let token = mio::Token(inner.listeners.len());
                    try!(inner.poll.register(&listener, token, mio::EventSet::readable(), mio::PollOpt::level())
                                   .map_err(|e| AddListenerError::Register {cause: e}));
                    inner.listeners.push(listener);
                    if inner.in_accept && !inner.fresh {
                        inner.fresh = true;
                        self.acceptor.inner_freshened.notify_one();
                    }
                    else {
                        inner.wake_buffer += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'a> Drop for AcceptorController<'a> {
    fn drop(&mut self) {
        let mut notify_write = self.acceptor.notify_write.lock().unwrap();
        // not much we could do about this
        let _ = notify_write.write(&[::NOTIFY_SHUTDOWN]);
    }
}
```

# Drawbacks

Needs to be implemented.

# Alternatives

We could instead build a transport abstraction layer around mioco or rotor.

# Unresolved Questions

None.

