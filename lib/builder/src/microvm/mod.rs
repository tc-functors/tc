use crate::types::BuildStatus;
use composer::Build;

use provider::Auth;
use provider::aws::microvm::MicroVmImage;
use provider::aws::microvm;


pub async fn build(auth: &Auth, _dir: &str, bspec: &Build) -> BuildStatus {

    // generate Dockerfile
    // zip
    // copy to s3

    let Build { image_name, base_image_arn, build_role_arn, uri, .. } = bspec;

    let client = microvm::make_client(auth).await;
    let mvi = MicroVmImage {
        name: image_name.to_string(),
        base_image_arn: base_image_arn.to_string(),
        build_role_arn: build_role_arn.to_string(),
        uri: uri.to_string()
    };
    let idem_token = format!("{}_{}", image_name, &auth.name);
    let image_id = mvi.find_or_create(&client, &idem_token).await;
    BuildStatus {
        path: image_id,
        status: true,
        out: String::from(""),
        err: String::from(""),
    }
}
