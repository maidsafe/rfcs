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

