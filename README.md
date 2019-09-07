# Enzyme 🧪

A bare bones http web framework that stays out of your way.  

## Endpoint

An endpoint is an async function from `Request` to `Response`.  An endpoint is
created using `Endpoint::new` taking an async route function and an async context
function as parameters.  The returned endpoint is a function that takes in a
`Request` and evaluates the route and context functions as steps in the request
evaluation.  

### Request and Response Types

Route functions must have the following signature:
```rust
async fn(ContextType, RequestType) -> WebResult<ResponseType>
```

`RequestType` and `ResponseType` can be any type that implements `Deserialize +
Default` and `Serialize` respectively.  The body of the request is deserialized
into what ever type is used as `RequestType` and the type used as `ResponseType`
is serialized into json automatically.  This means contract types are
automatically deserialized and serialized, you just define routes that consume
your `RequestType` and return your `ResponseType` and the framework does the
rest.  

### Context Types

`ContextType` can be any type, the async context function is used to construct
the `ContextType` and must have the following signature:

```rust
async fn(Parts) -> WebResult<ContextType>
```

`Parts` is the `http::request::Parts` type from the `http` crate and contains
everything in the request except the body.  The framework provides a default
context type and a default context type function called `Context` and
`default_context` respectively.  The default `Context` type simply wraps the
`Parts` type as a member.  A more elaborate `ContextType` could have members
such as `auth_token` and the async context function that constructs it could
parse out and evaluate the `auth_token` for validity before returning the
`ContextType`.  Finally access to the context of the request is accomplished by
passing the `ContextType` into the async route function.  

### Error Handling

Async functions used in the framework return a `WebResult` type.  The
`WebResult` is a `Result` type with the error parameter set to be a `WebError`.
A `WebError` is a message and an error code.  The message can be anything that
implements `Serialize` and the code is an `StatusCode` type from the `http`
crate.  
