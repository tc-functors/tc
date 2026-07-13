use crate::Role;
use compiler::{
    FunctionSpec,
    RuntimeSpec,
};
use kit as u;

fn find_parent_function_role(dir: &str) -> Option<String> {
    u::find_self_or_parent_file(dir, "roles/function.json")
}

pub fn collect_aux_files(
    infra_dir: &str,
    fspec: &FunctionSpec,
    rspec: Option<&RuntimeSpec>,
    role: &Role,
) -> Vec<String> {
    let mut out: Vec<String> = vec![];

    out.push(u::follow_path(&format!(
        "{}/roles/{}.json",
        infra_dir, &fspec.name
    )));

    if !role.path.is_empty() {
        out.push(role.path.clone());
    }

    if let Some(p) = find_parent_function_role(infra_dir) {
        out.push(p);
    }

    out.push(u::follow_path(&format!(
        "{}/vars/{}.json",
        infra_dir, &fspec.name
    )));

    if let Some(rs) = rspec {
        if let Some(p) = &rs.vars_file {
            out.push(u::follow_path(p));
        }
    }

    out.sort();
    out.dedup();
    out
}
