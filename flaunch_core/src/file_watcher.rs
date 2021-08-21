use notify::{watcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Default)]
pub struct FileWatcher {
    callbacks: Vec<Box<dyn Fn(PathBuf)>>,
}

impl std::fmt::Debug for FileWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileWatcher").finish()
    }
}

impl FileWatcher {
    // fn get_watchlist() -> &'static mut HashMap<Receiver<DebouncedEvent>, Box<Fn()>> {
    //     static watchlist: HashMap<Receiver<DebouncedEvent>, Box<Fn()>> = HashMap::new();
    //     &mut watchlist
    // }

    pub fn watch<T>(&mut self, dir: PathBuf, on_watch: T)
    where
        T: Fn(PathBuf) + 'static,
    {
        // let (tx, rx) = channel();
        // // Create a watcher object, delivering debounced events.
        // // The notification back-end is selected based on the platform.
        // let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        // watcher.watch(dir, RecursiveMode::Recursive).unwrap();
        self.callbacks.push(Box::new(on_watch));
    }
}
