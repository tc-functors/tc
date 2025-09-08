use crate::Manifest;

struct Job {

}


struct Workflow {


}


pub fn generate_config(records: &Vec<Manifest>) -> String {
    format!(r#"
version: 2.1

parameters:
  tc-deploy-snapshot-pipeline:
    type: boolean
    default: false
  api_call:
    type: boolean
    default: false

jobs:
  hello1:
    docker:
      - image: cimg/base:2025.08
    steps:
      - run: echo "hello world1"

  hello2:
    docker:
      - image: cimg/base:2025.08
    steps:
      - run: echo "hello world2"

  hello3:
    docker:
      - image: cimg/base:2025.08
    steps:
      - run: echo "hello world3"

workflows:
  all-hellos:
    jobs:
      - hello1
      - hello2
      - hello3

"#)
}
