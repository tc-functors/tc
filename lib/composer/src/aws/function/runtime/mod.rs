pub mod lambda;
pub mod microvm;
pub mod agentcore;
pub mod common;
pub mod layer;

pub use common::{FileSystem, Network};

use compiler::{FunctionSpec, function::Provider};
use configurator::Config;
pub use common::Runtime;

impl Runtime {
    pub fn new(
        dir: &str,
        t_infra_dir: &str,
        namespace: &str,
        fspec: &FunctionSpec,
        fqn: &str,
        _cspec: &Config,
    ) -> Runtime {
        let rspec = fspec.runtime.clone();

        let infra_dir = match &fspec.infra_dir {
            Some(p) => p.to_string(),
            None => common::as_infra_dir(dir, t_infra_dir),
        };

        match rspec {
            Some(r) => {
                match r.provider {
                    Some(Provider::Lambda) => lambda::make(dir, &infra_dir, &namespace, fqn, fspec, &r),
                    Some(Provider::MicroVm) => microvm::make(dir, &infra_dir, &namespace, fqn, fspec, &r),
                    _ =>  lambda::make(dir, &infra_dir, &namespace, fqn, fspec, &r)
                }

            }
            None => common::make_default(dir, &infra_dir, namespace, fqn, fspec),
        }
    }
}
