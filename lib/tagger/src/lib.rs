pub mod git;
mod changelog;
use notifier::RichText;
use kit as u;
use kit::*;

fn inc_patch(v: &str) -> String {
    let version = git::maybe_semver(v);
    let mut next = version.clone();
    next.increment_patch();
    format!("{}.{}.{}", next.major, next.minor, next.patch)
}

fn inc_minor(v: &str) -> String {
    let version = git::maybe_semver(v);
    let mut next = version.clone();
    next.increment_minor();
    format!("{}.{}.0", next.major, next.minor)
}

fn inc_major(v: &str) -> String {
    let mut version = git::maybe_semver(v);
    version.increment_major();
    format!("{}.0.0", version.major)
}

fn has_patch(v: &str) -> bool {
    let version = git::maybe_semver(v);
    version.patch > 0
}

fn add_suffix(v: &str, suffix: &str) -> String {
    let version = git::maybe_semver(v);
    let next = version.clone();
    format!("{}.{}.{}-{}", next.major, next.minor, next.patch, suffix)
}

fn dec_minor(v: &str) -> String {
    let version = git::maybe_semver(v);
    let next = version.clone();
    if next.minor > 0 {
        let minor = next.minor - 1;
        if minor == 0 {
            format!("{}.0.2", next.major)
        } else {
            format!("{}.{}.0", next.major, minor)
        }
    } else {
        format!("{}.{}.0", next.major, next.minor)
    }
}

fn current_stable_minor(v: &str) -> String {
    let version = git::maybe_semver(v);
    let next = version.clone();
    if next.minor > 0 {
        format!("{}.{}.0", next.major, next.minor)
    } else {
        format!("{}.{}.0", next.major, next.minor)
    }
}

fn changelog_since_last(prefix: &str, version: &str, has_suffix: bool) -> String {
    let prev_ver;
    if has_suffix {
        prev_ver = current_stable_minor(version);
    } else {
        prev_ver = dec_minor(version);
    }

    println!("{}", prev_ver);
    let curr_tag = format!("{}-{}", prefix, version);
    let prev_tag = format!("{}-{}", prefix, prev_ver);
    changelog::generate(&prev_tag, &curr_tag)
}

// git

fn should_tag(tag: &str) -> bool {
    let dir = u::pwd();
    let c1 = format!("git rev-parse {}", tag);
    let c2 = format!("git log -n 1 --format=%H .");
    let c1_out = u::sh(&c1, &dir);
    let c2_out = u::sh(&c2, &dir);

    if !c1_out.contains("fatal: ambiguous argument") {
        println!("tag: {c1_out} ({tag})");
    }
    println!("rev: {c2_out}");
    c1_out != c2_out
}

#[derive(Clone, Debug)]
pub struct Tag {
    pub prefix: String,
    pub parent: String,
    pub create: bool,
    pub version: String,
}

pub fn next_tag(prefix: &str, next: &str, suffix: &str) -> Tag {
    match next {
        "major" => {
            let git_ver = git::latest_version(prefix);
            Tag {
                parent: git_ver.clone(),
                prefix: s!(prefix),
                create: true,
                version: inc_major(&git_ver),
            }
        }

        "minor" => {
            let git_ver = git::latest_version(prefix);
            Tag {
                parent: git_ver.clone(),
                prefix: s!(prefix),
                create: has_patch(&git_ver),
                version: match suffix {
                    "default" => inc_minor(&git_ver),
                    _ => add_suffix(&git_ver, suffix),
                },
            }
        }

        "patch" => {
            let git_ver = git::latest_version(prefix);
            let current_tag = format!("{}-{}", prefix, &git_ver);
            Tag {
                parent: git_ver.clone(),
                prefix: s!(prefix),
                create: should_tag(&current_tag),
                version: match suffix {
                    "default" => inc_patch(&git_ver),
                    _ => add_suffix(&git_ver, suffix),
                },
            }
        }

        _ => Tag {
            parent: u::empty(),
            prefix: u::empty(),
            create: false,
            version: u::empty(),
        },
    }
}

fn fmt_msg(prefix: &str, version: &str, parent: &str, changes: &str) -> String {
    let title = &format!("QA Release | {} ", u::simple_date());
    let summary = &format!("*{}* - `{}` (annot on: {})", prefix, version, parent);
    let rt = RichText::new(title, summary, &changes);
    serde_json::to_string(&rt).unwrap()
}

async fn dry_run(next: &str, tag: Tag, has_suffix: bool) {
    let Tag {
        parent,
        prefix,
        version,
        ..
    } = tag;
    git::fetch_tags();

    match next {
        "patch" => {
            let tag = format!("{}-{}", &prefix, &version);
            let commit_msg = git::commit_message(&tag);
            println!("{}", commit_msg);
        }
        "minor" => {
            let changes = changelog_since_last(&prefix, &version, has_suffix);
            let msg = fmt_msg(&prefix, &version, &parent, &changes);
            notifier::slack(&prefix, msg).await;
        }
        _ => println!("Nothing to do.."),
    }
}

async fn create(next: &str, tag: Tag, push: bool, has_suffix: bool) {
    let Tag {
        parent,
        prefix,
        version,
        create,
        ..
    } = tag;

    match next {
        "patch" => {
            let tag = format!("{}-{}", &prefix, &version);
            if create {
                println!("Creating tag {}", &tag);
                git::create_tag(&tag, None);
                if push {
                    git::push_tag(&tag);
                    let commit_msg = git::commit_message(&tag);
                    let msg = format!("Created Patch Release {} -{}", tag, commit_msg);
                    notifier::slack(&prefix, notifier::wrap_msg(&msg)).await;
                }
            } else {
                println!("Not tagging or releasing {}", &tag);
            }
        }

        "minor" => {
            let tag = format!("{}-{}", prefix, version);

            if create {
                git::fetch_tags();
                let parent_tag = format!("{}-{}", prefix, parent);

                println!("Creating minor git tag {}", &tag);
                let parent_revision = git::tag_revision(&parent_tag);

                println!("{} {}", &tag, &parent_revision);
                git::create_annotated_tag(&tag, Some(parent_revision.clone()));

                if push {
                    git::push_tag(&tag);
                    let changes = changelog_since_last(&prefix, &version, has_suffix);
                    let msg = fmt_msg(&prefix, &version, &parent, &changes);
                    println!("{}", &msg);
                    notifier::slack(&prefix, msg.clone()).await;
                    notifier::slack("QA", msg).await;
                }
            } else {
                println!("Not creating {}", &tag);
            }
        }

        _ => println!("Nothing to do yet"),
    }
}

fn delete_current_minor(prefix: &str, version: &str) {
    let stable_version = current_stable_minor(version);
    let tag = format!("{}-{}", &prefix, &stable_version);
    let cmd = format!("git tag -d {} && git push --tag origin :{}", &tag, &tag);
    u::runcmd_stream(&cmd, &u::pwd());
}


// pub

pub async fn create_tag(next: &str, prefix: &str, suffix: &str, push: bool, is_dry_run: bool) {
    let tag = next_tag(&prefix, &next, &suffix);
    let has_suffix = suffix != "default";
    if is_dry_run {
        println!("dry: {:?}", tag);
        dry_run(&next, tag, has_suffix).await;
    } else {
        create(&next, tag, push, has_suffix).await;
    }
}

pub fn unwind(prefix: &str) {
    git::fetch_tags();
    let version = git::latest_version(prefix);
    delete_current_minor(prefix, &version);
}

pub fn changelog(namespace: &str, between: Option<String>, verbose: bool) {
    if u::option_exists(between.clone()) {
        changelog::between(namespace, between)
    } else {
        changelog::list(namespace, verbose);
    }
}

pub fn changelogs_since_last(prefix: &str, version: &str) -> String {
    let prev_ver = dec_minor(version);
    let curr_tag = format!("{}-{}", prefix, version);
    let prev_tag = format!("{}-{}", prefix, prev_ver);
    let from_sha = git::tag_revision(&prev_tag);
    let to_sha = git::tag_revision(&curr_tag);
    git::changelogs(&from_sha, &to_sha)
}

pub async fn find_version_history(namespace: &str, term: &str) -> Option<String> {
    let version = changelog::find_version(namespace, term);
    version
}
