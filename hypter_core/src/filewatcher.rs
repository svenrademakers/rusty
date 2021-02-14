use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn start_script_watcher(watch_dir: String) {
    let dir = std::path::Path::new(&watch_dir);
    if !dir.exists() {
        std::fs::create_dir(dir).unwrap();
    }

    let (tx, rx) = channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(3)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(watch_dir, RecursiveMode::Recursive).unwrap();

    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
