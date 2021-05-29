pub mod sx {

    use std::{
        borrow::{Borrow, BorrowMut},
        cell::{RefCell, RefMut},
        collections::{HashMap, HashSet},
        hash::Hash,
        ops::{Deref, DerefMut, Index, IndexMut},
        rc::Rc,
    };

    /// Global is an struct that can be used as an static global.
    /// It uses interoir mutability to modify its contents.
    /// use `Global::try_borrow_mut()` to access the data mutable.
    /// This function silences shared mutablity errors.
    /// in this case `Global::try_borrow_mut()` returns `None`
    /// # Example
    /// ```
    /// use sven_utils::sx::Global;
    ///
    /// static MYGLOBAL: Global<Vec<u32>> = Global::new(Vec::new());
    /// let mut test = MYGLOBAL.try_borrow_mut();
    /// assert!(test.is_some());
    /// //assert_eq!(0, (*&test.unwrap()).len());
    /// //let mut data = test.unwrap();
    /// //data.push(123);
    /// ````
    /// note: sychronisation is achieved using spinlocks.
    pub struct Global<T> {
        data: RefCell<T>,
        lock: spin::Mutex<()>,
    }

    impl<T> Global<T> {
        pub const fn new(value: T) -> Self {
            Global {
                data: RefCell::new(value),
                lock: spin::Mutex::new(()),
            }
        }

        /// Returns an wrapper object, `GlobalGuard`, which contains references
        /// to the containing data.
        /// locks until unique access is aquired using active spinning.
        pub fn try_borrow_mut<'a>(&'a self) -> Option<GlobalGuard<'a, T>> {
            match self.data.try_borrow_mut() {
                Ok(x) => Some(GlobalGuard::new(x, &self.lock)),
                Err(_) => None,
            }
        }
    }

    unsafe impl<T> Send for Global<T> {}
    unsafe impl<T> Sync for Global<T> {}

    pub struct GlobalGuard<'a, T> {
        data: RefMut<'a, T>,
        lock: &'a spin::Mutex<()>,
    }

    impl<'a, T> GlobalGuard<'a, T> {
        pub fn new(data: RefMut<'a, T>, lock: &'a spin::Mutex<()>) -> Self {
            lock.try_lock().unwrap();
            GlobalGuard {
                data: data,
                lock: lock,
            }
        }
    }

    impl<'a, T> Deref for GlobalGuard<'a, T> {
        type Target = T;
        fn deref(&self) -> &T {
            &*self.data
        }
    }

    impl<'a, T> DerefMut for GlobalGuard<'a, T> {
        fn deref_mut(&mut self) -> &mut T {
            &mut *self.data
        }
    }

    impl<'a, T> Drop for GlobalGuard<'a, T> {
        fn drop(&mut self) {
            unsafe { self.lock.force_unlock() };
        }
    }

    #[derive(Debug, Clone)]
    pub struct GenerationKey<T> {
        data: T,
        generation: u8,
    }

    // pub struct EntityMap<K> {
    //     map: HashMap<K, V>,
    // }

    // impl<K: Eq + Hash, V> EntityMap<K, V> {
    //     pub fn new() -> Self {
    //         EntityMap {
    //             map: HashMap::new(),
    //         }
    //     }

    //     pub fn insert(&mut self, key: K, value: V) {
    //         self.map.borrow_mut().insert(key, value);
    //     }

    //     pub fn get_keys(&self) -> Vec<K> {
    //         Vec::new()
    //     }

    //     pub fn create_transaction(&self) -> SyncTransaction<K, V> {
    //         SyncTransaction::new(&self.map)
    //     }
    // }
}
