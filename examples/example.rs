use std::time::Duration;

use futures::prelude::*;
use tokio_ctrlc_error::AsyncCtrlc;
use tokio_timer;

fn lengthy_task() -> impl Future<Item = (), Error = failure::Error> {
    tokio_timer::sleep(Duration::from_secs(5)).from_err()
}

fn main() {
    let task = lengthy_task().handle_ctrlc();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let res = rt.block_on(task);
    println!("{:?}", res);
}
