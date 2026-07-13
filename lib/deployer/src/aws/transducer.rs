use super::function::lambda;
use composer::{
    Function,
    Transducer,
};
use provider::Auth;
use std::collections::HashMap;

pub async fn create(
    auth: &Auth,
    fns: &HashMap<String, Function>,
    transducer: &Transducer,
    config: &HashMap<String, String>,
) {
    println!("Creating transducer");
    transducer.dump(config);

    let _ = builder::build(auth, &transducer.function, None, None, true).await;

    let client = lambda::make_client(auth).await;
    lambda::create(&client, &transducer.function, &HashMap::new()).await;

    for (name, f) in fns {
        if f.targets.len() > 0 {
            println!("Creating transducer destination for {}", name);
            lambda::update_destination(&client, &f.fqn, &transducer.arn).await;
        }
    }
    transducer.clean();
}

pub async fn delete(auth: &Auth, transducer: &Transducer) {
    let client = lambda::make_client(auth).await;

    lambda::delete(&client, &transducer.function).await;
}
