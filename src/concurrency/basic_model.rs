// Send and Sync property for smart pointers.
// Summary Table
// Smart Pointer	    Send condition	Sync condition	        Notes
// Box<T>	            T: Send	        T: Sync	                Simple heap pointer
// Rc<T>	            never	        never	                Non‑atomic reference count
// Arc<T>	            T: Send + Sync	T: Send + Sync	        Atomic reference count
// Weak<T>	            T: Send + Sync	T: Send + Sync	        Same as Arc, points to same allocation
// Cell<T>	            T: Send	        never	                No runtime checks; not thread‑safe to share
// RefCell<T>	        T: Send	        never	                Runtime borrow counter is non‑atomic
// Mutex<T>	            T: Send	        T: Send	                Lock provides synchronization
// RwLock<T>	        T: Send	        T: Send + Sync	        Multiple readers require T: Sync
// *const T / *mut T	never	        never	                Unsafe, must be explicitly opted in

