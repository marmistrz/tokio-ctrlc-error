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
use futures::prelude::*;
use std::marker::PhantomData;
use tokio_signal::{IoFuture, IoStream};

#[derive(Debug, Fail)]
#[fail(display = "keyboard interrupt")]
pub struct KeyboardInterrupt;

pub struct CtrlcAsError<F, E>
where
    F: Future,
    E: From<KeyboardInterrupt> + From<F::Error>,
{
    ctrlc: IoStream<()>,
    future: F,
    phantom: PhantomData<E>,
}

impl<F, E> CtrlcAsError<F, E>
where
    F: Future,
    E: From<KeyboardInterrupt> + From<F::Error>,
{
    fn from_future(future: F) -> Self {
        Self {
            ctrlc: Box::new(tokio_signal::ctrl_c().flatten_stream()),
            future,
            phantom: PhantomData,
        }
    }
}

impl<F, E> Future for CtrlcAsError<F, E>
where
    F: Future,
    E: From<KeyboardInterrupt> + From<F::Error>,
{
    type Error = E;
    type Item = F::Item;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let ctrlc_fut = self.ctrlc.poll().expect("ctrlc handling failed");
        if ctrlc_fut.is_ready() {
            Err(KeyboardInterrupt.into())
        } else {
            self.future.poll().map_err(Into::into)
        }
    }
}

pub trait FutureExt<F, E>
where
    F: Future,
    E: From<KeyboardInterrupt> + From<F::Error>,
{
    /// Intercept ctrl+c during execution and return an error in such case.
    fn ctrlc_as_error(self) -> CtrlcAsError<F, E>;
}

impl<F, E> FutureExt<F, E> for F
where
    F: Future<Error = failure::Error> + 'static,
    E: From<KeyboardInterrupt> + From<F::Error>,
{
    fn ctrlc_as_error(self) -> CtrlcAsError<F, E> {
        CtrlcAsError::from_future(self)
    }
}
