use failure::Fallible;
use futures::prelude::*;
use std::time::Duration;
use tokio_ctrlc_error::{FutureExt, KeyboardInterrupt};
use tokio_timer;

fn lengthy_task() -> impl Future<Item = (), Error = failure::Error> {
    tokio_timer::sleep(Duration::from_secs(5)).from_err()
}

fn main() {
    let task = lengthy_task().ctrlc_as_error();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let res: Fallible<_> = rt.block_on(task);
    if let Err(e) = res {
        match e.downcast::<KeyboardInterrupt>() {
            Ok(_) => println!("Keyboard interrupt!"),
            Err(_) => unreachable!(),
        }
    } else {
        println!("Timed out.")
    }
}
