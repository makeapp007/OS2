use std::{sync, thread};
use std::cmp::PartialEq;
use std::cmp::Ordering;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Mutex,Condvar};
use std::cell::UnsafeCell;


/// Provides a reader-writer lock to protect data of type `T`
pub struct RwLock<T> {
    // six terms
    data: UnsafeCell<T>,
    reader_cv: Condvar,
    writer_cv: UnsafeCell<Vec<Condvar>>,
    pref:Preference,
    order: Order,
    status: Mutex<Vec<i32>>,
    // vec0 reader active
    // vec1 reader waiting
    // vec2 write active
    // vec3 write waiting
}

#[derive(PartialEq)]
pub enum Preference {
    /// Readers-preferred
    /// * Readers must wait when a writer is active.
    /// * Writers must wait when a reader is active or waiting, or a writer is active.
    Reader,
    /// Writers-preferred: 
    /// * Readers must wait when a writer is active or waiting.
    /// * Writer must wait when a reader or writer is active.
    Writer,
}

/// In which order to schedule threads
#[derive(PartialEq)]
pub enum Order {
    /// First in first out
    Fifo,
    /// Last in first out
    Lifo,
}

impl<T> RwLock<T> {
    /// Constructs a new `RwLock`
    ///
    /// data: the shared object to be protected by this lock
    /// pref: which preference
    /// order: in which order to wake up the threads waiting on this lock
    pub fn new(data: T, pref: Preference, order: Order) -> RwLock<T> {
        let mut vec_status=vec![0; 4];
        RwLock{
            data: UnsafeCell::new(data),
            reader_cv: Condvar::new(),
            writer_cv: UnsafeCell::new(Vec::new()),
            pref: pref,
            order: order,
            status: Mutex::new(vec_status),

        }
    }

    /// Requests a read lock, waits when necessary, and wakes up as soon as the lock becomes available.
    /// 
    /// Always returns Ok(_).
    /// (We declare this return type to be `Result` to be compatible with `std::sync::RwLock`)
    pub fn read(&self) -> Result<RwLockReadGuard<T>, ()> {
        // check status, decide to wait or start to read
        // check preference
        if self.pref==Preference::Writer{
            {
                // lock the mutex
                let mut guard = self.status.lock().unwrap();
                if guard[2]>0 || guard[3]>0 { //autoderef
                    // active and waiting writer
                    guard[1]+=1;
                    guard=self.reader_cv.wait(guard).unwrap();
                    guard[1]-=1;
                }
                guard[0]+=1;
            } //drop the lock, calling mutex's drop trait
            // all writers finished
        }
        else{
            // reader prefer
            {
                let mut guard = self.status.lock().unwrap();
                if guard[2]>0{
                    // active writer
                    guard[1]+=1;
                    guard=self.reader_cv.wait(guard).unwrap();   
                    guard[1]-=1;
                }
                guard[0]+=1;
            }

        }
        // start to read
        // return read value
        
        // Ok(self)
        Ok(RwLockReadGuard{__lock:self})

    }

    /// Requests a write lock, and waits when necessary.
    /// When the lock becomes available,
    /// * if `order == Order::Fifo`, wakes up the first thread
    /// * if `order == Order::Lifo`, wakes up the last thread
    /// 
    /// Always returns Ok(_).
    pub fn write(&self) -> Result<RwLockWriteGuard<T>, ()> {
        // the argument is immutable, use unsafecell to modify data's value
        if self.pref==Preference::Writer{
            {
                // lock the mutex
                let mut guard = self.status.lock().unwrap();
                if guard[0]>0 || guard[2]>0 {
                    // active reader and writer 
                    guard[3]+=1;
                    // guard=self.reader_cv.wait(guard).unwrap();
                    let cv=Condvar::new();
                    // get the vec
                    let cv_get=unsafe{
                        &mut *(self.writer_cv.get())
                    };
                    cv_get.push(cv);
                    // wait
                    let length=cv_get.len();
                    guard=cv_get[length].wait(guard).unwrap();
                    guard[3]-=1;
                }
                guard[2]+=1;
            } //drop the lock, calling the drop trait
            // all writers finished
            // start to read
        }
        else{
            // reader prefer
            {
                let mut guard = self.status.lock().unwrap();
                if guard[2]>0 || guard[0]>0 || guard[1]>0{
                    // active reader and writer, and waiting reader
                    guard[3]+=1;
                    let cv=Condvar::new();
                    // get the vec
                    let cv_get=unsafe{
                        &mut *(self.writer_cv.get())
                    };
                    cv_get.push(cv);
                    // wait
                    let length=cv_get.len();
                    guard=cv_get[length].wait(guard).unwrap();

                    guard[3]-=1;
                }
                guard[2]+=1;
            }
        }
        // &self
        Ok(RwLockWriteGuard{__lock:self})
    }
}

/// Declares that it is safe to send and reference `RwLock` between threads safely
unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

/// A read guard for `RwLock`
pub struct RwLockReadGuard<'a, T: 'a> {
    __lock: &'a RwLock<T>,
}

/// Provides access to the shared object
impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    // when a lock is created, it will autoderef, and get the lock
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.__lock.data.get() }
    }

}

/// Releases the read lock
impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    // ...
    fn drop(&mut self) {
        println!("------------" );
        // unsafe { self.__lock.inner.read_unlock(); }
        // finish this reader
        // check Preference and decide to do notify or not
        if self.__lock.pref==Preference::Writer{
            let mut guard = self.__lock.status.lock().unwrap();
            // decrease active reader
            guard[0]-=1;
            if guard[0]==0 && guard[2]==0{
                // no active reader and writer 
                if guard[3]>0{
                    // if waiting writer exists,schedule it
                    guard[3]-=1;
                    if self.__lock.order==Order::Lifo{

                        let cv_get=unsafe{
                            &mut *(self.__lock.writer_cv.get())
                        };
                        println!("------------22");
                        match cv_get.pop(){
                            Some(x)=>x.notify_one(),
                            None=>{},
                        }
                        println!("------------22_continue");
                        // let cv=cv_get.pop().unwrap();
                        // println!("------------{:?}",cv );
                        // cv.notify_one();
                    }
                    else{
                        // Fifo
                        let cv_get=unsafe{
                            &mut *(self.__lock.writer_cv.get())
                        };

                        let cv=cv_get.remove(0);
                        cv.notify_one();
                    }
                }
            }
        }
        else{
            // reader prefer
            let mut guard = self.__lock.status.lock().unwrap();
            // decrease active reader
            guard[0]-=1;
            if guard[0]==0 && guard[1]==0 && guard[2]==0{
                // no active reader and writer and no waiting reader
                if guard[3]>0{
                    // if waiting writer exists
                    guard[3]-=1;
                    if self.__lock.order==Order::Lifo{
                        let cv_get=unsafe{
                            &mut *(self.__lock.writer_cv.get())
                        };
                        // let cv=cv_get.pop().unwrap();
                        // cv.notify_one();
                        match cv_get.pop(){
                            Some(x)=>x.notify_one(),
                            None=>{},
                        }

                    }
                    else{
                        // Fifo
                        let cv_get=unsafe{
                            &mut *(self.__lock.writer_cv.get())
                        };
                        let cv=cv_get.remove(0);
                        cv.notify_one();
                    }
                }
            }

        }
    }
}

/// A write guard for `RwLock`
pub struct RwLockWriteGuard<'a, T: 'a> {
    __lock: &'a RwLock<T>,
}

/// Provides access to the shared object
impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.__lock.data.get() }
    }

}

/// Provides access to the shared object
impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    // ...
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.__lock.data.get() }
    }

}

/// Releases the write lock
impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    // ...
    fn drop(&mut self) {
        // finish this writer
        if self.__lock.pref==Preference::Writer{
            // writer prefer
            let mut guard = self.__lock.status.lock().unwrap();
            // decrease active writer
            guard[2]-=1;
            if guard[3]>0{
                // if exists waiting writer
                guard[3]-=1;
                if self.__lock.order==Order::Lifo{
                    let cv_get=unsafe{
                        &mut *(self.__lock.writer_cv.get())
                    };
                    let cv=cv_get.pop().unwrap();
                    cv.notify_one();

                }
                else{
                    // Fifo
                    let cv_get=unsafe{
                        &mut *(self.__lock.writer_cv.get())
                    };
                    let cv=cv_get.remove(0);
                    cv.notify_one();
                }
            }
            else{
                // if waiting reader and no waiting writer
                if guard[1]>0{
                    guard[0]=guard[1];
                    guard[1]=0;
                    // notify all
                    self.__lock.reader_cv.notify_all();
                }
            }
        }
        else{
            // reader prefer
            let mut guard = self.__lock.status.lock().unwrap();
            // decrease active reader
            guard[2]-=1;
            if guard[1]>0{
                // if exists waiting reader
                guard[0]=guard[1];
                guard[1]=0;
                self.__lock.reader_cv.notify_all();
            }
            else{
                // if exists waiting writer
                if guard[3]>0{
                    guard[3]-=1;
                    if self.__lock.order==Order::Lifo{
                        let cv_get=unsafe{
                            &mut *(self.__lock.writer_cv.get())
                        };
                        let cv=cv_get.pop().unwrap();
                        cv.notify_one();
                    }
                    else{
                        // Fifo
                        let cv_get=unsafe{
                            &mut *(self.__lock.writer_cv.get())
                        };
                        let cv=cv_get.remove(0);
                        cv.notify_one();
                    }
                }
            }

        }        
    }

}

