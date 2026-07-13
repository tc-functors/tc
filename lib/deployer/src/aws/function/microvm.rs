use composer::{
    Function,
};

use provider::{
    Auth,
    aws::microvm,
    aws::microvm::MicroVm
};
use std::collections::HashMap;

fn make_vm(auth: &Auth, f: &Function, image_id: &str) -> MicroVm {

    if let Some(cfg) = &f.runtime.microvm {

        MicroVm {
            image_id: image_id.to_string(),
            role: auth.role_arn(&f.runtime.role.name),
            ingress_network_connectors: cfg.ingress_network_connectors.clone().unwrap(),
            egress_network_connectors: cfg.egress_network_connectors.clone().unwrap(),
            max_duration: cfg.max_duration.unwrap(),
            log_group: None,
            idle_policy: String::from("")
        }
    } else {
        panic!("No microvm default network connectors found")
    }
}

pub async fn create(auth: &Auth, f: &Function, _tags: &HashMap<String, String>) -> String {
    let client = microvm::make_client(auth).await;
    let image_name = f.build.image_name.clone();
    let maybe_image_id = microvm::find_image(&client, &image_name).await;
    if let Some(image_id) = maybe_image_id {
        let maybe_microvm = microvm::find_by_image_id(&client, &image_id).await;
        if let Some(microvm_id) = maybe_microvm {
            println!("Found running microvm: {}", &microvm_id);
            if microvm::is_suspended(&client, &microvm_id).await {
                println!("Resuming microvm: {}", &microvm_id);
                microvm::resume(&client, &microvm_id).await;
                microvm_id.to_string()
            } else {
                microvm_id.to_string()
            }
        } else {
            let mv = make_vm(auth, f, &image_id);
            let run_info = mv.run(&client).await;
            println!("id: {}", &run_info.microvm_id);
            run_info.microvm_id.to_string()
        }
    } else {
        panic!("No image found {}, exiting", &image_name);
    }
}

pub async fn delete(auth: &Auth, function: &Function, force: bool) {
    let client = microvm::make_client(auth).await;
    let image_name = function.build.image_name.clone();
    let maybe_microvm = microvm::find(&client, &image_name).await;
    if let Some(microvm_id) = maybe_microvm {
        if force {
            println!("Terminating function microvm: {}", &function.name);
            microvm::terminate(&client, &microvm_id).await;
        } else {
            println!("Suspending function microvm: {}", &function.name);
            microvm::suspend(&client, &microvm_id).await;
        }
    } else {
        println!("No microvm found");
    }
    if force {
        println!("Deleting microvm image {}", &image_name);
        microvm::delete_image(&client, &image_name).await;
    }
}

pub async fn print_config(auth: &Auth, function: &Function) {
    let client = microvm::make_client(auth).await;
    let image_name = function.build.image_name.clone();
    let maybe_microvm = microvm::find(&client, &image_name).await;
    if let Some(microvm_id) = maybe_microvm {
        let run_info = microvm::get_microvm(&client, &microvm_id).await;
        println!("microvm_id: {}", run_info.microvm_id);
        println!("");

        if let Some(token) = microvm::get_token(&client, &microvm_id, 30).await {

            println!("To run the microvm: ");
            println!("curl https://{} -X 'x-aws-proxy-auth: {}'", run_info.endpoint, &token);
            println!("");
            println!("");
    }
    }
}
