use std::collections::VecDeque;
use std::error::Error;
use std::sync::Arc;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Address;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::HttpUrl;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::Cookie::*;
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::connection::ReusePlan::*;

// Forward declaration of RealConnection as it is used in the trait signatures.
// In a real project, this would be imported from its actual module.
pub trait RealConnection: Send + Sync {}

// Policy on choosing which connection to use for an exchange and any retries that follow.
//
// Implementations of this trait are not thread-safe. Each instance is thread-confined to the
// thread executing the call.
pub trait RoutePlanner {
    fn address(&self) -> &Address;

    // Follow-ups for failed plans and plans that lost a race.
    fn deferred_plans(&self) -> &VecDeque<Arc<dyn RoutePlanner::Plan>>;

    fn is_canceled(&self) -> bool;

    // Returns a plan to attempt.
    fn plan(&self) -> Result<Arc<dyn RoutePlanner::Plan>, Box<dyn Error + Send + Sync>>;

    // Returns true if there's more route plans to try.
    //
    // @param failed_connection an optional connection that was resulted in a failure.
    fn has_next(&self, failed_connection: Option<Arc<dyn RealConnection>>) -> bool;

    // Returns true if the host and port are unchanged from when this was created.
    fn same_host_and_port(&self, url: &HttpUrl) -> bool;

    // A plan holds either an immediately-usable connection, or one that must be connected first.
    pub trait Plan: Send + Sync {
        fn is_ready(&self) -> bool;

        fn connect_tcp(&self) -> RoutePlanner::ConnectResult;

        fn connect_tls_etc(&self) -> RoutePlanner::ConnectResult;

        fn handle_success(&self) -> Arc<dyn RealConnection>;

        fn cancel(&self);

        // Returns a plan to attempt if canceling this plan was a mistake!
        fn retry(&self) -> Option<Arc<dyn RoutePlanner::Plan>>;
    }

    // What to do once a plan has executed.
    //
    // If `next_plan` is not-null, another attempt should be made by following it.
    // If `throwable` is non-null, it should be reported to the user should all further attempts fail.
    #[derive(Debug, Clone)]
    pub struct ConnectResult {
        pub plan: Arc<dyn RoutePlanner::Plan>,
        pub next_plan: Option<Arc<dyn RoutePlanner::Plan>>,
        pub throwable: Option<Arc<dyn Error + Send + Sync>>,
    }

    impl ConnectResult {
        pub fn is_success(&self) -> bool {
            self.next_plan.is_none() && self.throwable.is_none()
        }
    }
}