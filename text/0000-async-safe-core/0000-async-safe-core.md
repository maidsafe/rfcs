# Async `safe_core`

- Status: proposed
- Type: Refactor
- Related components: `safe_core`
- Start Date: 19-07-2016
- Discussion: (fill me in with link to RFC discussion - shepherd will complete this)
- Supersedes: (fill me in with a link to RFC this supersedes - if applicable)
- Superseded by: (fill me in with a link to RFC this is superseded by - if applicable)

## Summary

A proposal to allow non-blocking APIs within the `safe_core` crate.

## Motivation

Current design needs threads to allow concurrency. However, the main user of the
`safe_core` API is JavaScript code, which is a language not very friendly
towards threads. After the refactor, the main user of the `safe_core` crate will
be able to do several concurrent requests without waiting for the reply to the
first request.

## Detailed design

This API is inspired by the SDL event system, which share common guidelines with
other non-blocking C APIs (with C expressive language you cannot go much
further). I've must surely been influenced by other APIs as well (e.g. EFL).

The bottom of this API is to provide an event queue from which you can receive
the results of the pending operations.

More elaborate and general APIs like EFL provide an _extensible_ foundation from
which you can also create non-blocking functions on the top of the same API you
use to consume the the async results provided by the library. In contrast,
simple and non-extensible APIs like SDL provide a few async functions and an API
to extract the results that you cannot use yourself to deliver the results of
_your_ async functions.

SDL's async foundation limitations aren't important to our use case, as the user
of the `safe_core` crate will use it only for the `core`, `nfs` and `dns`
abstractions and nothing else. In fact, JavaScript code (the main user of
`safe_core`) will use its own libraries to implement other features and manage
other async actions.

Actually, there was a simplification done til now that happens to be a lie. And
now it's the time to discuss this matter. SDL actually doesn't expose async
functions. You don't "initiate" an async operation in SDL. What SDL provides are
events. When you start using SDL, it's like you're already requesting all
actions that trigger these events. Therefore, this proposal goes just a little
more complex to give support for actual async functions.

FFI modules of some languages don't play well interacting with custom C
structs. Therefore, we avoid them here.

The full API is given below:

```c
typedef void safe_event_t;

int safe_poll_ev(safe_event_t **ev);
int safe_get_ev_id(safe_event_t *ev);
void* safe_get_ev_data(safe_event_t *ev);
void safe_delete_ev(safe_event_t *ev);
```

`safe_event_t` is an opaque type that glues together an event id that you also
get when you call the initiating function for some async operation, a pointer to
some data that you must cast to the appropriate type -- which depends on the
event type -- and a deleter to such data that you must call when you're done
with the event.

You use `safe_poll_ev` to query for new events. If there is an event in the
queue, `1` is returned and the pointer pointed by `ev` is modified to store the
event removed from the queue. `0` is returned otherwise.

`safe_ev_get_id` returns the id associated with an event. This id is generated
when you schedule a new operation and it is guaranteed it won't be reused until
you're done with this event (i.e. call `safe_delete_ev`). It's a concept akin to
unique tokens and fds.

You can use `safe_get_ev_data` to receive an opaque pointer that can contain
extra data associated with the event. Each type of operation should have its own
type for the result of the operation and this type should be used uniformly
across all calls to such operation.

You must call `safe_delete_ev` when you're done with the event. This will free
the extra data hold by the data pointer and free the id to be reused in other
operations. If you think you'll need to have the result laying around for a long
time, you should copy all the data you need to your own structures and then free
this event, as it is crucial to free ids to new operations.

And that's the whole API.

### Example

Suppose you want to convert current `create_account` API to
non-blocking. Currently it looks like this:

```rust
unsafe extern fn create_account(c_pass_phrase: *const c_char, ffi_handle: *mut *mut FfiHandle) -> int32_t;
```

You must split the action of scheduling/initiating the operation and
gathering/extracting the result. The initiating operation could look like this:

```c
// Returns the event ID
int create_account(const char *pass_phrase);
```

And the following functions to extract the result:

```c
// Returns the errno-like object containing the status of the completion
int32_t create_account_result(void *data);

// Returns the ffi_handle
FfiHandle* create_account_result_handle(void *data);
```

The `data` argument can be discovered calling `safe_get_ev_data`.

## Drawbacks

- This API doesn't take into account multiple threads. It assumes an implicit
  global variable (the queue), which is kind of ugly.
- This API doesn't add many safe guards for the sake of being simpler (e.g. an
  event type tag is not present).

## Alternatives

C language is the de franco lingua for FFI and there isn't much space for
expressiveness in such API. Therefore, there weren't many alternatives that were
taken into account.

The main alternative would be a callback-based API, but this was rejected as it
might poses complication when interacting with the JavaScript layer.

## Unresolved questions

This is the foundation RFC. It describes the foundation API and guidelines to
convert existing `safe_core` blocking API. A second RFC with the new API might
be needed.
