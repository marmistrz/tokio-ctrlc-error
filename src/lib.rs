//! Easy ctrl+c handling with `failure` and `futures`.
//!
//! In many cases, a ctrl+c event from the user is hardly different from
//! a fatal application error. This crate, inspired by Python's `InterruptedException`
//! makes it easy to treat ctrl+c in precisely such a way
//!
//! # Examples
//! ```
//!     use std::time::Duration;
//!     use futures::prelude::*;
//!     use tokio_ctrlc_error::AsyncCtrlc;
//!
//!     fn lengthy_task() -> impl Future<Item = (), Error = failure::Error> {
//!         futures::future::ok(())
//!     }
//!
//!     let task = lengthy_task().handle_ctrlc();
//!     let mut rt = tokio::runtime::Runtime::new().unwrap();
//!     let res = rt.block_on(task);
//!     println!("{:?}", res);
//! ```

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

pub trait FutureExt<F: Future> {
    /// Intercept ctrl+c during execution and return an error in such case.
    fn ctrlc_as_error(self) -> CtrlcAsError<F>;
}

impl<F: Future> FutureExt<F> for F
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
    use super::FutureExt;
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
