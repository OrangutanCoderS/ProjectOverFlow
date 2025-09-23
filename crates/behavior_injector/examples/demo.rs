//! Example usage of behavior_injector
//! Shows how to build an InterventionRequest and execute safely.

use behavior_injector::{
    dispatch_via_global, InterventionRequest, install_global_injector,
    InjectorConfig, BehaviorInjector, Action,
};

fn main() {
    // Install global injector (dry-run for demo)
    let config = InjectorConfig { dry_run: true, ..Default::default() };
    let injector = BehaviorInjector::new_default(config).unwrap();
    install_global_injector(injector);

    // Build a request with the new constructor
    let req = InterventionRequest::new(Action::ActionKill, Some(99999), None);

    // Dispatch via global injector
    match dispatch_via_global(req) {
        Ok(result) => println!("Intervention result: {:?}", result),
        Err(err) => eprintln!("Error: {:?}", err),
    }
}
