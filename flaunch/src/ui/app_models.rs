use std::rc::{Rc, Weak};

use flaunch_core::{
    logging::error,
    script_engine::{Script, ScriptChange},
};
use futures::{channel::mpsc::Receiver, executor::block_on, select, FutureExt, StreamExt};

pub struct AppDataModels {
    script_listeners: Vec<Weak<dyn ScriptEngineModelObserver>>,
    script_receiver: Receiver<ScriptChange>,
}

impl AppDataModels {
    pub fn new(script_receiver: Receiver<ScriptChange>) -> Self {
        AppDataModels {
            script_listeners: Vec::new(),
            script_receiver: script_receiver,
        }
    }

    pub fn subscribe_script_engine<T: 'static + ScriptEngineModelObserver>(&mut self, who: &Rc<T>) {
        let weak = Rc::downgrade(who);
        // ugh.. cant compare weaks
        // if self.script_listeners.contains(&weak) {
        //     return;
        // }

        self.script_listeners.push(weak);
    }

    pub fn poll(&mut self) {
        let poll_models = async {
            select! {
                update = self.script_receiver.select_next_some() => self.notify_script_observers(update),
                default => (),
            };
        };
        block_on(poll_models.fuse());
    }

    fn notify_script_observers(&mut self, update: ScriptChange) {
        match update {
            ScriptChange::NewOrUpdated(x) => {
                AppDataModels::execute(&mut self.script_listeners, |obs| {
                    obs.new_or_updated(&x);
                });
            }
            ScriptChange::Deleted(key) => {
                AppDataModels::execute(&mut self.script_listeners, |obs| {
                    obs.removed(key);
                });
            }
        }
    }

    fn execute<T: ?Sized, F>(list: &mut Vec<Weak<T>>, func: F)
    where
        F: Fn(&T),
    {
        for n in (0..list.len()).rev() {
            if let Some(sub) = list[n].upgrade() {
                func(sub.as_ref());
            } else {
                list.remove(n);
            }
        }
    }
}

pub trait ScriptEngineModelObserver {
    fn new_or_updated(&self, scripts: &Vec<Script>);
    fn removed(&self, key: u64);
}
