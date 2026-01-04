use std::sync::LazyLock;
use std::thread::ThreadId;

static MAIN_THREAD_ID: LazyLock<ThreadId> = LazyLock::new(|| std::thread::current().id());
pub fn ensure_main_thread() -> cu::Result<()> {
    if *MAIN_THREAD_ID != std::thread::current().id() {
        cu::bail!(
            "unexpected: this operation should only be called on the main thread - this is an internal bug"
        );
    }
    Ok(())
}

// Guard invariants:
// - object always on main thread
// - ALIVE = true when object is alive
// - no other reference exists
macro_rules! main_thread_singleton {
    () => {};
    (__impl__ __const__ $type:ty, $init:block) => {
        static mut INSTANCE: $type = $init;
        impl Guard {
            /// SAFETY: must be called on main thread
            #[inline(always)]
            unsafe fn new_main_thread() -> cu::Result<Self> {
                let p = &raw mut INSTANCE;
                // SAFETY: address of static variable can't be null
                Ok(Self(unsafe { std::ptr::NonNull::new_unchecked(p) }))
            }
        }
    };
    (__impl__ __non_const__ $type:ty, $init:block) => {
        static mut INSTANCE: Option<$type> = None;
        impl Guard {
            /// SAFETY: must be called on main thread
            #[inline(always)]
            unsafe fn new_main_thread() -> cu::Result<Self> {
                let initialized = {
                    // SAFETY: short-lived
                    unsafe{&*(&raw const INSTANCE)}.is_some()
                };
                if !initialized {
                    let init_value: cu::Result<$type> = (|| { $init })();
                    let init_value = init_value?;
                    {
                        // SAFETY: we are on main thread, only main thread can call this
                        unsafe { INSTANCE = Some(init_value) }
                    }
                }
                let inner_ptr = {
                    std::ptr::NonNull::from(
                        // SAFETY: short-lived reference
                        unsafe{&*(&raw const INSTANCE)}.as_ref().unwrap()
                    )
                };
                Ok(Self(inner_ptr))
            }
        }
    };
    (__impl__ mod $constness:ident, $xxx:ident, $type:ty, $init:block, $($rest:tt)* ) => {
        mod $xxx {
            #[allow(unused)]
            use super::*;
            static ALIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            pub(crate) struct Guard(std::ptr::NonNull<$type>);
            impl std::ops::Deref for Guard {
                type Target = $type;
                fn deref(&self) -> &Self::Target {
                    let p = self.0.as_ptr();
                    // SAFETY: invariants
                    unsafe {&*p}
                }
            }
            impl std::ops::DerefMut for Guard {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    let p = self.0.as_ptr();
                    // SAFETY: invariants
                    unsafe {&mut *p}
                }
            }
            // invariant: object always on main thread
            $crate::internal::main_thread_singleton!(__impl__ $constness $type, $init);
            impl Drop for Guard {
                fn drop(&mut self) {
                    // relaxed is fine because we check it's only used on the main thread
                    ALIVE.store(false, std::sync::atomic::Ordering::Relaxed);
                }
            }
            /// Get instance - will error if not on the main thread or another Guard is alive
            pub fn instance() -> cu::Result<Guard> {
                use cu::Context as _;
                crate::internal::ensure_main_thread().context(concat!(stringify!($xxx), " instance can only be accessed on the main thread"))?;
                // relaxed is fine because we check it's only used on the main thread
                cu::ensure!(
                    ALIVE.compare_exchange(
                        false, true,
                        std::sync::atomic::Ordering::Relaxed,
                        std::sync::atomic::Ordering::Relaxed,
                    ).is_ok(),
                    concat!("another guard of ", stringify!($xxx), " is alive")
                );
                // SAFETY: ensured on main thread
                let result = unsafe { Guard::new_main_thread() };
                result.context(concat!("failed to initialize main thread singleton: ", stringify!($xxx)))
            }
        }
        $crate::internal::main_thread_singleton!($($rest)*);
    };
    (const $xxx:ident = $type:ident :: $init_ident:ident (); $($rest:tt)* ) => {
        $crate::internal::main_thread_singleton!(__impl__ mod __const__, $xxx, $type, { $type::$init_ident() }, $($rest)*);
    };
    (let $xxx:ident = $type:ident :: $init_ident:ident (); $($rest:tt)* ) => {
        $crate::internal::main_thread_singleton!(__impl__ mod __non_const__, $xxx, $type, { $type::$init_ident() }, $($rest)*);
    };
}
pub(crate) use main_thread_singleton;
