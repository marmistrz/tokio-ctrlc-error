# tokio-ctrlc-error 
[![Build Status](https://travis-ci.org/marmistrz/tokio-ctrlc-error.svg?branch=master)](https://travis-ci.org/marmistrz/tokio-ctrlc-error)
[![Docs](https://docs.rs/tokio-ctrlc-error/badge.svg)](https://docs.rs/tokio-ctrlc-error)
[![crates-io-badge]][crates-io]

[crates-io-badge]: https://img.shields.io/badge/crates.io-v0.1.0-orange.svg?longCache=true
[crates-io]: https://crates.io/crates/tokio-ctrlc-error

Easy ctrl+c handling with `failure` and `futures`.

In many cases, a ctrl+c event from the user is hardly different from
a fatal application error. This crate, inspired by Python's `InterruptedException`
makes it easy to treat ctrl+c in precisely such a way.

## Docs
[API Documentation (Releases)](https://docs.rs/tokio-ctrlc-error/0.1.0/tokio_ctrlc_error/)

## Examples
```
    use futures::prelude::*;
    use tokio_ctrlc_error::AsyncCtrlc;

    fn lengthy_task() -> impl Future<Item = (), Error = failure::Error> {
        futures::future::ok(())
    }

    let task = lengthy_task().ctrlc_as_error();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(task);
    println!("{:?}", res);
```

## Usage notes
`ctrlc_as_error` has the same semantics as `select` and will return either
the result of the future or an `KeyboardInterrupt` error, whichever occurs
first. In particular, the interrupt is intercepted **only for those futures
in the chain that precede the call**. For example:

```
    use std::time::Duration;
    use futures::prelude::*;
    use tokio_ctrlc_error::AsyncCtrlc;

    fn sleep() -> impl Future<Item = (), Error = failure::Error> {
        // The sleep is very short, so that the tests don't take too much time
        tokio_timer::sleep(Duration::from_millis(1)).from_err()
    }

    let task = sleep()
        .ctrlc_as_error()
        .and_then(|_| sleep());
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(task);
```

Here, the interrupt will be handled only during the first sleep.
During the second sleep, the default handling of the signal will take place.
