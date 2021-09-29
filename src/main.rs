use std::time::{Duration, Instant};

use simple_executor::{executor::new_executor_and_spawner, timer::TimerFuture};

fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        println!("{:?}", Instant::now());
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("{:?}", Instant::now());
    });

    drop(spawner);

    executor.run();
}

