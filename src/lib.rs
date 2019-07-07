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

#[derive(Debug, Fail)]
#[fail(display = "interrupt received")]
pub struct InterruptedError;

pub trait AsyncCtrlc<F>
where
    F: Future,
{
    /// Intercept ctrl+c during execution and return an error in such case.
    fn handle_ctrlc(self) -> Box<dyn Future<Item = F::Item, Error = F::Error> + Send>;
}

impl<F> AsyncCtrlc<F> for F
where
    F: Future<Error = failure::Error> + 'static + Send,
    F::Item: Send,
{
    fn handle_ctrlc(self) -> Box<dyn Future<Item = F::Item, Error = F::Error> + Send> {
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
                Either::B(_) => Err(InterruptedError.into()),
            });
        Box::new(fut)
    }
}
