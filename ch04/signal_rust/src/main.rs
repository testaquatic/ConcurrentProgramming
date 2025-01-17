use std::{process, thread, time::Duration};

use signal_hook::{consts::SIGUSR1, iterator::Signals};

fn main() -> Result<(), anyhow::Error> {
    println!("pid: {}", process::id());

    let mut signals = Signals::new([SIGUSR1])?;
    thread::spawn(move || {
        signals
            .forever()
            .for_each(|sig| println!("received signal: {:?}", sig));
    });

    thread::sleep(Duration::from_secs(10));

    Ok(())
}
