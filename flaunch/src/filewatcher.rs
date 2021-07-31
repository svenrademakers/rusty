use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn start_script_watcher(
    engine: Rc<RefCell<ScriptEngine>>,
    settings_changed: watch::Receiver<SettingsChanged>,
    watch_dir: PathBuf,
) -> notify::Receiver {
    let context = glib::MainContext::default();

    let handle_watcher = async move {
    let (tx, rx) = channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(watch_dir, RecursiveMode::Recursive).unwrap();
    };
    
}
