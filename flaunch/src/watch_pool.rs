//use std::rc::{Rc, Weak};

use tokio::sync::watch;

// pub struct WatchPool<T> {
//     watchers: Vec<Weak<dyn UpdateWatcher<T>>>,
// }

// impl<T> Default for WatchPool<T>
// where
//     T: 'static,
// {
//     fn default() -> Self {
//         WatchPool {
//             watchers: Vec::new(),
//         }
//     }
// }

// impl<T> WatchPool<T>
// where
//     T: 'static,
// {
//     pub fn start_watching(&self, receiver: watch::Receiver<T>) {
//         let watchers = self.watchers.clone();
//         let update_func = move |change: &T| {
//             let mut stale = 0;

//             for n in 0..watchers.len() {
//                 match watchers[n].upgrade() {
//                     Some(w) => w.change(&change),
//                     None => stale += 1,
//                 }
//             }

//             !(stale == watchers.len())
//         };

//         watch(receiver, update_func);
//     }

//     pub fn add_watcher<W>(&mut self, watcher: &Rc<W>)
//     where
//         W: 'static + UpdateWatcher<T>,
//     {
//         // // cleanup
//         // if let Some(x) = Rc::get_mut(&mut self.to_delete) {
//         //     x.reverse();
//         //     for i in x {
//         //         self.watchers.remove(i);
//         //     }
//         // }

//         let rc: Rc<dyn UpdateWatcher<T>> = watcher.clone();

//         self.watchers.push(Rc::downgrade(&rc));
//     }
// }

// pub trait UpdateWatcher<T> {
//     fn change(&self, change: &T);
// }

pub fn watch<T, F>(mut receiver: watch::Receiver<T>, mut closure: F)
where
    F: FnMut(&T) -> bool + 'static,
    T: 'static,
{
    let future = async move {
        loop {
            if let Ok(_) = receiver.changed().await {
                if !closure(&*receiver.borrow()) {
                    return;
                }
            } else {
                return;
            }
        }
    };

    glib::MainContext::default().spawn_local(future);
}
