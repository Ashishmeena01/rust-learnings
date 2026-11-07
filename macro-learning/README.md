# Procedural macro example

This learning project contains the `macrolib1` attribute-macro crate and a small binary that uses it. From this directory, run the demonstration with:

```sh
cargo run -p macro-test-1
```

The library currently provides `log_call` and `test_fn1` attributes.

## `log_call`

Place `#[log_call]` above a function to print its name immediately before the original function body runs. The attribute leaves the function's signature and body intact, so the function keeps its normal arguments and return value.

## `test_fn1`

Place `#[test_fn1]` above a function to print `hello` and then run that function's original body. It is a deliberately small example of parsing an `ItemFn`, replacing its block, and emitting the updated function.
