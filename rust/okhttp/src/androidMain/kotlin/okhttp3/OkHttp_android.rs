use std::sync::OnceLock;

// Mocking the external dependencies based on the provided Kotlin source and context.
// In a real production environment, these would be imported from the respective crates.
pub mod internal {
    pub const CONST_VERSION: &str = "4.12.0"; // Example version string

    pub mod platform {
        use std::sync::Mutex;

        // Mocking android.content.Context
        #[derive(Debug, Clone, PartialEq)]
        pub struct Context {
            pub name: String,
        }

        impl Context {
            pub fn application_context(&self) -> Context {
                // In Android, this returns the application context
                self.clone()
            }
        }

        pub struct PlatformRegistry;

        impl PlatformRegistry {
            // Using Mutex for thread-safe interior mutability to mimic the Kotlin object property
            pub static APPLICATION_CONTEXT: Mutex<Option<Context>> = Mutex::new(None);
        }
    }
}

use crate::internal::CONST_VERSION;
use crate::internal::platform::{Context, PlatformRegistry};

// OkHttp singleton equivalent.
pub struct OkHttp;

impl OkHttp {
    // The version of OkHttp.
    pub const VERSION: &'static str = CONST_VERSION;

    // Configure the ApplicationContext. Not needed unless the AndroidX Startup [Initializer] is disabled, or running
    // a robolectric test.
    //
    // The functionality that will fail without a valid Context is primarily Cookies and URL Domain handling, but
    // may expand in the future.
    pub fn initialize(application_context: Context) {
        let mut ctx_lock = PlatformRegistry::APPLICATION_CONTEXT.lock().unwrap();
        if ctx_lock.is_none() {
            // Make sure we aren't using an Activity or Service Context
            // Kotlin: PlatformRegistry.applicationContext = applicationContext.applicationContext
            *ctx_lock = Some(application_context.application_context());
        }
    }
}

// To maintain the "object" singleton behavior in Rust, we can provide a global instance 
// or simply use the associated functions of the OkHttp struct as shown above.