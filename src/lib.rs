//! Easy ctrl+c handling with `failure` and `futures`.
//!
//! In many cases, a ctrl+c event from the user is hardly different from
//! a fatal application error. This crate, inspired by Python's `InterruptedException`
//! makes it easy to treat ctrl+c in precisely such a way.
//!
//!
//! # Examples
//! ```
//!     use futures::prelude::*;
//!     use tokio_ctrlc_error::AsyncCtrlc;
//!
//!     fn lengthy_task() -> impl Future<Item = (), Error = failure::Error> {
//!         futures::future::ok(())
//!     }
//!
//!     let task = lengthy_task().ctrlc_as_error();
//!     let mut rt = tokio::runtime::Runtime::new().unwrap();
//!     let res = rt.block_on(task);
//!     println!("{:?}", res);
//! ```
//!
//! # Usage notes
//! `ctrlc_as_error` has the same semantics as `select` and will return either
//! the result of the future or an `KeyboardInterrupt` error, whichever occurs
//! first. In particular, the interrupt is intercepted **only for those futures
//! in the chain that precede the call**. For example:
//!
//! ```
//!     use std::time::Duration;
//!     use futures::prelude::*;
//!     use tokio_ctrlc_error::AsyncCtrlc;
//!
//!     fn sleep() -> impl Future<Item = (), Error = failure::Error> {
//!         // The sleep is very short, so that the tests don't take too much time
//!         tokio_timer::sleep(Duration::from_millis(1)).from_err()
//!     }
//!
//!     let task = sleep()
//!         .ctrlc_as_error()
//!         .and_then(|_| sleep());
//!     let mut rt = tokio::runtime::Runtime::new().unwrap();
//!     let res = rt.block_on(task);
//! ```
//!
//! Here, the interrupt will be handled only during the first sleep.
//! During the second sleep, the default handling of the signal will take place.

use failure::Fail;
use futures::{prelude::*, FlattenStream};
use tokio_signal::{IoFuture, IoStream};

#[derive(Debug, Fail)]
#[fail(display = "keyboard interrupt")]
pub struct KeyboardInterrupt;

#[derive(Debug, Fail)]
#[fail(display = "I/O error handling ctrl+c: {}", _0)]
pub struct IoError(std::io::Error);

pub struct CtrlcAsError<F> {
    // we will switch to `struct CtrlC` in tokio 0.3
    ctrlc: FlattenStream<IoFuture<IoStream<()>>>,
    future: F,
}

impl<F: Future> Future for CtrlcAsError<F>
where
    F::Error: From<KeyboardInterrupt> + From<IoError>,
{
    type Error = F::Error;
    type Item = F::Item;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let ctrlc_fut = self.ctrlc.poll().map_err(IoError)?;
        if ctrlc_fut.is_ready() {
            Err(KeyboardInterrupt.into())
        } else {
            self.future.poll().map_err(Into::into)
        }
    }
}

pub trait AsyncCtrlc<F: Future> {
    /// Intercept ctrl+c during execution and return an error in such case.
    fn ctrlc_as_error(self) -> CtrlcAsError<F>;
}

impl<F: Future> AsyncCtrlc<F> for F
where
    F::Error: From<KeyboardInterrupt> + From<IoError>,
{
    fn ctrlc_as_error(self) -> CtrlcAsError<F> {
        CtrlcAsError {
            ctrlc: tokio_signal::ctrl_c().flatten_stream(),
            future: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncCtrlc;
    use futures::prelude::*;

    // Test if it compiles when used with the multi-threaded runtime
    #[test]
    fn test_send_future() {
        use tokio::runtime::Runtime;
        fn get_fut() -> Box<dyn Future<Item = (), Error = failure::Error> + Send> {
            let f = futures::future::ok(());
            Box::new(f)
        }

        let future = get_fut().ctrlc_as_error();
        let mut rt = Runtime::new().unwrap();
        rt.block_on(future).unwrap();
    }

    // Test if it compiles when used with the single-threaded runtime
    #[test]
    fn test_non_send_future() {
        use tokio::runtime::current_thread::Runtime;
        fn get_fut() -> Box<dyn Future<Item = (), Error = failure::Error>> {
            let f = futures::future::ok(());
            Box::new(f)
        }

        let future = get_fut().ctrlc_as_error();
        let mut rt = Runtime::new().unwrap();
        rt.block_on(future).unwrap();
    }

}
