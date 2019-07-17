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
use futures::{FlattenStream, prelude::*};
use tokio_signal::{IoFuture, IoStream};

#[derive(Debug, Fail)]
#[fail(display = "keyboard interrupt")]
pub struct KeyboardInterrupt;

pub struct CatchCtrlC<F> {
    work: F,
    ctrl_c: FlattenStream<IoFuture<IoStream<()>>>,
}

pub trait AsyncCtrlc<F>
where
    F: Future,
{
    /// Intercept ctrl+c during execution and return an error in such case.
    fn handle_ctrlc(self) -> CatchCtrlC<F>;
}

impl<F: Future + 'static> AsyncCtrlc<F> for F {
    fn handle_ctrlc(self) -> CatchCtrlC<F> {
        let ctrl_c = tokio_signal::ctrl_c().flatten_stream();
        let work = self;

        CatchCtrlC { work, ctrl_c }
    }
}

impl<F: Future> Future for CatchCtrlC<F>
where
    F::Error: From<KeyboardInterrupt> + From<std::io::Error>,
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match (self.work.poll(), self.ctrl_c.poll()) {
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e.into()),
            (Ok(Async::Ready(v)), _) => Ok(Async::Ready(v)),
            (_, Ok(Async::Ready(_))) => Err(KeyboardInterrupt.into()),
            (Ok(Async::NotReady), Ok(Async::NotReady)) => Ok(Async::NotReady),
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

        let future = get_fut().handle_ctrlc();
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

        let future = get_fut().handle_ctrlc();
        let mut rt = Runtime::new().unwrap();
        rt.block_on(future).unwrap();
    }

}
