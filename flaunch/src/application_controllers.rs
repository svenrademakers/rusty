use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use flaunch_core::logging::error;
use once_cell::sync::OnceCell;
use tokio::sync::mpsc;
use tokio::sync::watch;

static CONTROLLERS: OnceCell<std::sync::Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>> =
    OnceCell::new();

pub fn watch<T, F>(mut closure: F)
where
    F: FnMut(&T) -> bool + 'static,
    T: 'static + std::fmt::Debug + Sync,
{
    let f = async {
        if let Some(mut receiver) = get_internal::<watch::Receiver<T>>().await {
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
        } else {
            error!(
                "no watcher initialized for type{:?}",
                TypeId::of::<watch::Receiver<T>>()
            );
        }
    };
    glib::MainContext::default().spawn_local(f);
}

pub fn control<T>(val: T)
where
    T: 'static + std::fmt::Debug + Sync,
{
    let fut = async {
        if let Some(sender) = get_internal::<mpsc::Sender<T>>().await {
            sender.send(val).await.unwrap();
        } else {
            error!(
                "controller not initialized for type {:?}",
                TypeId::of::<mpsc::Sender<T>>()
            );
        }
    };

    glib::MainContext::default().spawn_local(fut);
}

async fn get_internal<T: 'static + Clone>() -> Option<T> {
    let controllers = CONTROLLERS.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
    let map = controllers.lock().unwrap();
    if let Some(ctrl) = (*map).get(&TypeId::of::<T>()) {
        return Some(ctrl.as_ref().downcast_ref::<T>().unwrap().clone());
    }
    None
}

pub fn register_sender_receiver<T: 'static + Send + Sync>(controller: T) {
    let mut controllers = CONTROLLERS
        .get_or_init(|| std::sync::Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    (*controllers).insert(TypeId::of::<T>(), Box::new(controller));
}
