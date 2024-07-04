# Async safe_core
- Status: implemented
- Type: feature
- Related components: `safe_core`, `safe_launcher`
- Start Date: 02-September-2016
- Discussion: https://forum.safedev.org/t/rfc-43-async-safe-core/99

## Summary
Making `safe_core` async with respect to FFI and internally.

## Conventions
- The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](http://tools.ietf.org/html/rfc2119).

## Motivation
Currently `safe_core` has a threaded design. It has properly mutexed core-objects and parallel invocations can be achieved by making calls from different threads. However the frontend coded in NodeJS interfaces via `NodeFFI` which detects the hardware concurrency and limits the number of threads to that. This is not bad but leads to underutilisation of `safe_core`. This RFC proposes a design in which number of invocations are not limited. While this will lead to optimal usage of `safe_core` itself, it can cause very high network traffic, so we will try and address such problems too.

## Detailed design
Let's assume the hardware concurrency is 2 (dual core) for a machine. In current threaded design, if `App_0` makes 2 requests to `Launcher`, `App_1` makes 2, `NodeFFI` module in `Launcher` queues 2 of them and sends only 2 to `safe_core` spawing 2 threads for parallelism. Until atleast one thread returns, none of the 2 queued requests will have any chance of getting through to `safe_core`. `safe_core` invariably waits tremendous amount of time (in terms of CPU speeds) for network responses (_IO_). While it sits doing nothing it could have handled so many more requests if only `NodeFFI` forwarded it. This leads to underutilisation of the core library. However there are positives to this. For instance if `NodeFFI` were to spawn a thread per request, then we could take that to the other extreme saying that 30 combined requests from a few apps will result in 30 threads which is also not good. Further, by restricting number of simultaneous invocations the throttling mechanism can be view as in-built. One would not be allowed to choke the network/bandwidth by making 100's of concurrent requests. So we need to strike a balance.

To better utilise the library without uncontrolled spawning of resource hogging threads, this RFC proposes async design involving single threaded event loop. We will use `futures-rs` crate. There will be a central event loop running on a single thread which registers futures and dispatches them when ready.

The current code, completely stripped down, can be approximated as follows:
```rust
pub struct FfiHandle {
    client: Arc<Mutex<Client>>,
}

#[no_mangle]
pub unsafe extern "C" fn foo(handle: *const FfiHandle) -> *const i8 { // blocking call
    let mut client = (*handle).client.clone();

    let rx = {
        let mut client_guard = client.lock().unwrap();
        client_guard.foo()
    };

    // Client is now free to take more requests (we have released the mutex).

    let result = rx.recv().unwrap(); // <<<<------- This is a blocking wait (Code-point-0).

    transform_appropriately(result)
}

pub struct Client { ... };
impl Client {
    pub fn foo(&mut self) -> mpsc::Receiver<ResponseEvent> {
        let uuid = get_uuid();
        let (tx, rx) = mpsc::channel();
        let rx = register_observer(uuid, tx);

        self.routing.network_request_foo(uuid);

        rx
    }
}

pub type ObserverQueue = HashMap<UUID, mpsc::Sender<ResponseEvent>>;

pub fn register_observer(uuid: UUID, observer: mpsc::Sender<ResponseEvent>) {
    observer_queue.lock().unwrap().insert(uuid, observer);
}

fn listen_to_routing() {
    thread::spawn(move || {
        for it in routing_rx.iter() {
            match it {
                RoutingEvent::FooResult(uuid, result) => {
                    let response_event = ResponseEvent::Foo(result);
                    let tx = observer_queue.lock().unwrap().remove(uuid);
                    tx.send(response_event).unwrap();
                }
            }
        }
    }).unwrap();
}
```

If multiple apps need to call FFI `foo()` (or a single app calls it multiple times) simultaneously, Launcher spawns a thread per call as stated before. The underutilisation of the library can be seen from the fact that though `Client` is free to handle more task as stated by inline comment in snippet above, `NodeFFI` will limit the number of simultaneous calls to FFI `foo` to hardware concurrency of the system. Time is wasted doing nothing at `Code-point-0`.

In the proposed design, we will not have any blocking calls, thus `NodeFFI` invocation of FFI `foo` will returns immediately and it is free to call it as many times it can. The difference would be the way result is returned. In the approach listed above, since it was a blocking call the result of `*const i8` was conveniently returned by `foo` itself. In async design using event loop and future based mechanism, we will return even if the result is not immediately available. So we accept a callback instead which we will invoke once we do have the result. The above code will be transformed as follows:
```rust
pub struct FfiHandle {
    client: Arc<Mutex<Client>>,
}

#[no_mangle]
pub unsafe extern "C" fn foo(handle: *const FfiHandle,
                             o_cb: extern "C" fn(u64, *const i8)) -> u64 { // async call
    let mut client = (*handle).client.clone();

    let rx = {
        let mut client_guard = client.lock().unwrap();
        client_guard.foo()
    };

    let uuid_u64 = generate_uuid_u64();

    rx.map(move |result| {
        o_cb(transform_appropriately(uuid_u64, result));
    }).forget();

    // Code no longer blocks and returns immediately.
    // Launcher can tie in the uuid when callback is invoked.
    uuid_u64
}

pub struct Client { ... };
impl Client {
    pub fn foo(&mut self) -> futures::OneShot<ResponseEvent> {
        let uuid = get_uuid();
        let (tx, rx) = futures::oneshot(); // In master.
                                           // Called futures::promise() in crate.io
        let rx = register_observer(uuid, tx);

        self.routing.network_request_foo(uuid);

        rx
    }
}

pub type ObserverQueue = HashMap<UUID, futures::Complete<ResponseEvent>>;

pub fn register_observer(uuid: UUID, observer: futures::Complete<ResponseEvent>) {
    observer_queue.lock().unwrap().insert(uuid, observer);
}

fn listen_to_routing() {
    thread::spawn(move || {
        for it in routing_rx.iter() {
            match it {
                RoutingEvent::FooResult(uuid, result) => {
                    let response_event = ResponseEvent::Foo(result);
                    let tx = observer_queue.lock().unwrap().remove(uuid);
                    tx.complete(response_event);
                }
            }
        }
    }).unwrap();
}
```

Since `foo` is no longer blocking, it can be called unlimited times (limited only by resources) concurrently.

We might ask the question why we are going for an extra crate and types instead of using plain callbacks (`FnOnce`, `FnMut`) or traits, both of which are language features available without cost and could achieve the same goal without the overhead of learning to use an entire new library. The reason we chose future is that combinators such as `and_then` reduce the number of indentations compared to callbacks. To analyse this, consider the following blocking call (such patterns are present in `safe_core` so it's not just a theoretical example):
```rust
fn f0() -> Type0 { // blocking call
    let type_1 = f1(); // blocking call
    let type_2 = f2(); // blocking call
    let type_3 = f3(); // blocking call

    let type_0 = transform(type_1, type_2, type_3);

    type_0
}

fn f1() -> Type1 {
    let network_result_type = network_call(); // blocking call
    let type_1 = transform(network_result_type);
    type_1
}

fn f2() -> Type2 {
    let network_result_type = network_call(); // blocking call
    let type_2 = transform(network_result_type);
    type_2
}

fn f3() -> Type3 {
    let network_result_type = network_call(); // blocking call
    let type_3 = transform(network_result_type);
    type_3
}
```

A callback based solution would look like:
```rust
fn f0<F: FnOnce(Type0)>(cb: F) { // async call
    f1(move |type_1| {
        f2(move |type_2| {
            f3(move |type_3| {
                cb(transform(type_1, type_2, type_3));
            }
        }
    }
}

fn f1<F: FnOnce(Type1)>(cb: F) {
    network_call(move |network_result_type| { // Async call
        cb(transform(network_result_type));
    });
}

fn f2<F: FnOnce(Type2)>(cb: F) {
    network_call(move |network_result_type| { // async call
        cb(transform(network_result_type));
    });
}

fn f3<F: FnOnce(Type3)>(cb: F) {
    network_call(move |network_result_type| { // async call
        cb(transform(network_result_type));
    });
}
```
The level of indentation in functions like `f0` are directly proportional to number of blocking calls in the sync-counterpart in the previous (blocking) code. Here it is 3, but if it called 5 functions the level of indentations would be 5.

In future based approach we get the following refactor:
```rust
fn f0<F: FnOnce<Type0>)(cb: F) {
    let future_type_1 = f1(); // async call
    let future_type_2 = f2(); // async call
    let future_type_3 = f3(); // async call
    let joined_futures = future_type_1
        .join(future_type_2)
        .join(future_type_3);

    joined_futures.map(|((type_1, type_2), type_3)| {
        cb(transform(type_1, type_2, type_3));
    }).forget();
}

fn f1() -> Box<Future<Item=Type1, Error=futures::Canceled>> {
    let network_result_future = network_call(); // async call
    network_result_future.and_then(move |network_result_type| {
        let type_1 = transform(network_result_type);
        futures::finished(type_1)
    }).boxed()
}

fn f2() -> Box<Future<Item=Type2, Error=futures::Canceled>> {
    let network_result_future = network_call(); // async call
    network_result_future.and_then(move |network_result_type| {
        let type_2 = transform(network_result_type);
        futures::finished(type_2)
    }).boxed()
}

fn f3() -> Box<Future<Item=Type3, Error=futures::Canceled>> {
    let network_result_future = network_call(); // async call
    network_result_future.and_then(move |network_result_type| {
        let type_3 = transform(network_result_type);
        futures::finished(type_3)
    }).boxed()
}
```
As can be seen, there is no callback nesting.

### Throttle Implementation
Since `safe_core` would be completely async, it could end up get large number of requests from apps. While this is ideal from `safe_core`'s and apps' p.o.v., it has the potential to choke the network with request-flooding. Thus a throttling mechanism must be used.

There shall be a routing wrapper for for handling of `PUT, POST, GET, DELETE` requests. Instead of the requests going directly to routing it shall pass through this wrapper.
```rust
pub const MAX_LOAD_PER_AUTHORITY: usize = 10;

pub enum QueuedRequest {
    Get {
        auth: Authority,
        data_id: DataIdentifier,
        msg_id: MessageId,
    },
    Put {
        auth: Authority,
        data: Data,
        msg_id: MessageId,
    },
    Post {
        auth: Authority,
        data: Data,
        msg_id: MessageId,
    },
    Delete {
        auth: Authority,
        data: Data,
        msg_id: MessageId,
    },
}

pub struct Throttle {
    // RefCell due to API conformance with Routing - e.g. send_put requires non-mut self
    current_load: RefCell<HashMap<Authority, usize>>,
    // RefCell due to API conformance with Routing - e.g. send_put requires non-mut self
    pipeline: RefCell<HashMap<Authority, VecDeque<QueuedRequest>>>,
    routing: routing::Client,
}

impl Throttle {
    pub fn new(routing: routing::Client) -> Self;

    pub fn send_get_request(&mut self,
                            auth: Authority,
                            data_id: DataIdentifier,
                            msg_id: MessageId) -> Result<(), InterfaceError> {
        if *self.current_load.borrow().get(&auth) <= MAX_LOAD_PER_AUTHORITY {
            let res = self.routing.send_get_request(auth, data_id, msg_id);
            if res.is_ok() {
                let mut current_load = self.current_load.borrow_mut();
                let load = current_load.entry(auth).or_insert(0);
                *load += 1;
            }

            res
        } else {
            let req = QueuedRequest::Get {
                auth: auth,
                data_id: data_id,
                msg_id: msg_id,
            };
            let mut pipeline = self.pipeline.borrow_mut();
            let req_queue = pipeline.entry(auth).or_insert_with(|| VecDeque::new());
            req_queue.push(req);

            Ok(())
        }
    }

    pub fn send_put_request(&self,
                            auth: Authority,
                            data: Data,
                            msg_id: MessageId) -> Result<(), InterfaceError> {
        if *self.current_load.borrow().get(&auth) <= MAX_LOAD_PER_AUTHORITY {
            let res = self.routing.send_put_request(auth, data, msg_id);
            if res.is_ok() {
                let mut current_load = self.current_load.borrow_mut();
                let load = current_load.entry(auth).or_insert(0);
                *load += 1;
            }

            res
        } else {
            let req = QueuedRequest::Put {
                auth: auth,
                data: data,
                msg_id: msg_id,
            };
            let mut pipeline = self.pipeline.borrow_mut();
            let req_queue = pipeline.entry(auth).or_insert_with(|| VecDeque::new());
            req_queue.push(req);

            Ok(())
        }
    }

    // Similarly for Post and Delete

    pub fn on_authority_response(&mut self, responding_auth: Authority) {
        {
            let mut current_load = self.current_load.borrow_mut();
            if let Entry::Occupied(mut oe) = current_load.entry(auth) {
                *oe.get_mut() -= 1;
                if *oe.get() == 0 {
                    oe.remove();
                }
            }
        }

        let req = {
            let mut pipeline = self.pipeline.borrow_mut();
            if let Entry::Occupied(mut oe) = pipeline.entry(responding_auth) {
                if let Some(req) = oe.pop_front() {
                    if oe.is_empty() {
                        oe.remove();
                    }
                    req
                } else {
                    return oe.remove();
                }
            } else {
                return;
            }
        };

        match req {
            QueuedRequest::Get { auth, data_id, msg_id } => self.send_get_request(auth, data_id, msg_id),
            QueuedRequest::Put { auth, data, msg_id } => self.send_put_request(auth, data, msg_id),
            QueuedRequest::Post { auth, data, msg_id } => self.send_post_request(auth, data, msg_id),
            QueuedRequest::Delete { auth, data, msg_id } => self.send_delete_request(auth, data, msg_id),
        }
    }
}

impl ops::Deref for Throttle {
    type Target = routing::Client;

    fn deref(&self) -> &Self::Target {
        &self.routing
    }
}

impl ops::DerefMut for Throttle {
    fn deref(&mut self) -> &mut Self::Target {
        &mut self.routing
    }
}
```
Whenever routing responds it gives us the authority. We shall call `Throttle::on_authority_response` to see if there is any queued request corresponding to that authority and if so we release it.

The throttle limit shall be set to **10 per Authority**.

## Drawbacks
- As noted [here](https://github.com/alexcrichton/futures-rs/blob/0dc6e2563f2da56ad9de067a1686127b9230c2d1/TUTORIAL.md#returning-futures), currently there is no clean way to return a `Future` from a function. This suffers from the same disadvantages as `Iterators` traits. We use heap-allocated boxed return to simplify the function signature. Other methods mentioned in the link are uglier (as noted in the link too). Thus until we have [impl Trait](https://github.com/alexcrichton/futures-rs/blob/0dc6e2563f2da56ad9de067a1686127b9230c2d1/TUTORIAL.md#impl-trait) implemented in stable-rust we will have to live with the workaround if we follow this approach.

## Alternatives
- Instead of using `Futures`, use plain old callbacks. The downside to callbacks is the level of indentation with increase in number of calls. However if our use case does not have too many calls then the level of indentation might be OK. Also level of indentation is completely avoided if we delegate the code inside the closure to separate functions, at the expense of sacrificing the convenience of parameter-type-deduction and variable-capture from surrounding scope that closures provide. The second downside to using callbacks is the inverted flow-control. If these are not seen as great deterrents then callback based approach can be used in which case there is no need for external complex crates like `futures-rs`.
