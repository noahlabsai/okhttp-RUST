use std::io::{Error, ErrorKind};
use crate::okhttp::src::commonJvmAndroid::kotlin::okhttp3::internal::connection::{ExchangeFinder, RealConnection, RoutePlanner};

/// Attempt routes one at a time until one connects.
pub struct SequentialExchangeFinder {
    pub route_planner: Box<dyn RoutePlanner>,
}

impl SequentialExchangeFinder {
    pub fn new(route_planner: Box<dyn RoutePlanner>) -> Self {
        Self { route_planner }
    }
}

impl ExchangeFinder for SequentialExchangeFinder {
    fn route_planner(&self) -> &dyn RoutePlanner {
        self.route_planner.as_ref()
    }

    fn find(&self) -> Result<RealConnection, std::io::Error> {
        let mut first_exception: Option<std::io::Error> = None;

        loop {
            if self.route_planner.is_canceled() {
                return Err(std::io::Error::new(ErrorKind::Other, "Canceled"));
            }

            // We wrap the logic in a closure or block to simulate the try-catch behavior
            let result = (|| -> Result<Option<RealConnection>, std::io::Error> {
                let plan = self.route_planner.plan();

                if !plan.is_ready() {
                    let tcp_connect_result = plan.connect_tcp()?;
                    
                    let connect_result = if tcp_connect_result.is_success() {
                        plan.connect_tls_etc()?
                    } else {
                        tcp_connect_result
                    };

                    let (_, next_plan, failure) = connect_result;

                    if let Some(fail) = failure {
                        return Err(fail);
                    }
                    
                    if let Some(next) = next_plan {
                        // Note: route_planner.deferred_plans is assumed to be a 
                        // thread-safe collection (like Mutex<VecDeque>) given the context
                        self.route_planner.deferred_plans().push_front(next);
                        return Ok(None); // Signal to 'continue' the loop
                    }
                }
                
                Ok(Some(plan.handle_success()))
            })();

            match result {
                Ok(Some(connection)) => return Ok(connection),
                Ok(None) => continue,
                Err(e) => {
                    if first_exception.is_none() {
                        first_exception = Some(e);
                    } else {
                        // Rust's std::io::Error doesn't have a direct 'addSuppressed' 
                        // like Java's Throwable. In a production system, one might 
                        // use a custom error wrapper or a crate like 'anyhow'.
                        // To preserve behavior, we keep the first error and 
                        // effectively ignore subsequent ones or log them.
                    }

                    if !self.route_planner.has_next() {
                        return Err(first_exception.expect("first_exception must be set if has_next is false"));
                    }
                }
            }
        }
    }
}