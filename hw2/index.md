# Homework 2: Reader-writer lock

## Update

2017-03-22: Remove the requirement to use nightly Rust.

## Introduction

Implement a reader-writer lock based on mutexes (`std::sync::Mutex`) and condition variables (`std::sync::Condvar`). You will learn how to

* implement writers-preferred vs. readers-preferred policies;

* schedule threads in FIFO vs. LIFO policies;

* implement `Deref` and `DerefMut` to provide coercion between pointer types;

* implement `Drop` to automatically reset the state before leaving a critical section;

* use `UnsafeCell` to circumvent Rust's static aliasing rules safely at runtime;

## Public API

Your program must provide the following public API.

```rust
/// Provides a reader-writer lock to protect data of type `T`
pub struct RwLock<T> {
    ...
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
        ...
    }

    /// Requests a read lock, waits when necessary, and wakes up as soon as the lock becomes available.
    /// 
    /// Always returns Ok(_).
    /// (We declare this return type to be `Result` to be compatible with `std::sync::RwLock`)
    pub fn read(&self) -> Result<RwLockReadGuard<T>, ()> {
        ...
    }

    /// Requests a write lock, and waits when necessary.
    /// When the lock becomes available,
    /// * if `order == Order::Fifo`, wakes up the first thread
    /// * if `order == Order::Lifo`, wakes up the last thread
    /// 
    /// Always returns Ok(_).
    pub fn write(&self) -> Result<RwLockWriteGuard<T>, ()> {
        ...
    }
}

/// Declares that it is safe to send and reference `RwLock` between threads safely
unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

/// A read guard for `RwLock`
pub struct RwLockReadGuard<'a, T: 'a> {
    ...
}

/// Provides access to the shared object
impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = T;

    ...
}

/// Releases the read lock
impl<'a, T> Drop for RwLockReadGuard<'a, T> {
    ...
}

/// A write guard for `RwLock`
pub struct RwLockWriteGuard<'a, T: 'a> {
    ...
}

/// Provides access to the shared object
impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    ...
}

/// Provides access to the shared object
impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    ...
}

/// Releases the write lock
impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    ...
}
```

## Academic integrity

This is an individual homework. While you may share ideas, algorithms, and test cases with others, under no circumstances may you exchange solution code with others. Examples of cheating include (but are not limited to):

* Read or possess solution code written by other people, including people outside this course.
* Submit to the gradebot code written by other people or derived from the code written by other people.
* Allow other people to read or possess your solution code either intentionally or negligently, e.g., by posting your code on a web site or leaving the computer containing your code unattended and unlocked. You are responsible for exercising due diligence in safeguarding your code.

## Submission

```
$ cargo new rwlock
$ cd rwlock
$ # Save your program in `src/lib.rs`
$ git add Cargo.toml src/lib.rs
$ git commit
$ git remote add origin metastasis@gradebot.org:user/{username}/4/2
$ git push origin master
```

## Bonuses

* If your program passes all the test cases by Tuesday March 28, you will get 25% extra credit.

Last updated: 2017-03-22
