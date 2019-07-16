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
use futures::{future::Either, prelude::*};
use tokio_signal::{IoStream, IoFuture};
use std::marker::PhantomData;

#[derive(Debug, Fail)]
#[fail(display = "keyboard interrupt")]
pub struct KeyboardInterrupt;

struct CtrlcAsError<F, E> where F: Future, E: From<KeyboardInterrupt> {
    ctrlc: IoStream<()>,
    future: F,
    phantom: PhantomData<E>
}

impl<F, E> CtrlcAsError<F, E> where F: Future, E: From<KeyboardInterrupt> {
    fn from_future(future: F) -> Self {
        Self {
            ctrlc: Box::new(tokio_signal::ctrl_c().flatten_stream()),
            future,
            phantom: PhantomData
        }
    }
}

impl<F, E> Future for CtrlcAsError<F, E>  where F: Future, E: From<KeyboardInterrupt> {
    type Item = ();
    type Error = E;
    fn poll(&mut self) {

    }
}

/*
pub trait FutureExt<F>
where
    F: Future,
{
    /// Intercept ctrl+c during execution and return an error in such case.
    fn ctrlc_as_error(self) -> CtrlcAsError<F>;
}

impl<F> FutureExt<F> for F
where
    F: Future<Error = failure::Error> + 'static + Send,
    F::Item: Send,
{
    fn ctrlc_as_error(self) -> Box<dyn Future<Item = F::Item, Error = F::Error> + Send> {
        let ctrlc = tokio_signal::ctrl_c()
            .flatten_stream()
            .into_future()
            .map_err(|_| ());

        let fut = self
            .select2(ctrlc)
            .map_err(|e| match e {
                Either::A((e, _)) => e,
                _ => panic!("ctrl+c handling failed"),
            })
            .and_then(|res| match res {
                Either::A((r, _)) => Ok(r),
                Either::B(_) => Err(KeyboardInterrupt.into()),
            });
        Box::new(fut)
    }
}*/
