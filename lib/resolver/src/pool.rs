use super::{
    Context,
    Topology,
};
use composer::topology::Pool;
use std::collections::HashMap;

pub async fn resolve(ctx: &Context, topology: &Topology) -> HashMap<String, Pool> {
    let Context { config, auth, .. } = ctx;
    let mut pools: HashMap<String, Pool> = HashMap::new();

    for (name, mut pool) in topology.pools.clone() {
        let email_map = &config.aws.cognito.from_email_address_map;
        let env = &auth.name;

        let email = match email_map {
            Some(c) => {
                if let Some(p) = c.get(env) {
                    p
                } else {
                    &pool.from_email
                }
            }
            None => &pool.from_email,
        };

        pool.from_email = email.to_owned();
        pools.insert(name, pool);
    }
    pools
}
