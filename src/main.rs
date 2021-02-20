use azure_rs::pull_requests::{PullListOptions, PullRequestResponse, PullUpdateOptions};
use azure_rs::{ApiVersion, AzureClient, Credentials, Result};
use std::env::var;
use std::fs;
use std::process::Command;

extern crate clap;
use clap::{App, Arg};
use std::path::Path;

use std::fs::File;
use std::io::{BufWriter, Read, Write};
fn main() -> Result<()> {
    let check_remote = init_clap();

    if check_remote {
        let _ = get();
    }

    Ok(())
}

/// cargo run -- --config <TokenHere>
/// ./draft_push --config 12
pub fn init_clap() -> bool {
    let matches = App::new("Draft push")
        .version("1.0")
        .author("Behxhet S. <bensadiku65@gmail.com>")
        .about("Auto draft/publish pr after making changes")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .required(false)
                .help("Configures token that will be used for Azure")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("alias")
                .short("a")
                .long("alias")
                .required(false)
                .help("Configures xpush alias")
                .takes_value(true),
        )
        .get_matches();
    // Gets a value for config if supplied by user, or defaults to "null"
    // in which case, check if we have a token locally
    let config = matches.value_of("config").unwrap_or("null");

    // To be able to configure the alias within the binary
    let alias_path = matches.value_of("alias").unwrap_or("null");

    if config == "null" {
        if !Path::new(&get_cfg_home()).exists() {
            panic!("\nNo token and no file containg token\nUse ./draft_push --config <token>\n\n")
        } else {
            let token = get_token();
            if token.is_empty() {
                panic!("\nNo token in file \nUse ./draft_push --config <token>\n\n")
            }
        }
    } else {
        // Store token
        write_token(config);
        println!("Token configured");

        // If we don't have an alias setup, setup the default one
        // we do this because we don't want to overwrite the alias in case there's a custom one set
        if !does_alias_exist() {
            println!("Setting default alias");
            set_alias(None);
        }
        return false;
    }

    // If an alias has been supplied, set it.
    // TODO, check if alias is valid
    if alias_path != "null" {
        println!("Setting custom alias");
        set_alias(Some(alias_path));
        return false;
    }
    return true;
}

/// Configures the xpush git alias as global with the provided path
/// If no path is provided, it will default to "~/./draft_push"
/// git config --global alias.xpush '!git push $1 $2 && ~/./draft_push'
pub fn set_alias(path: Option<&str>) {
    let path = if path.is_none() {
        "~/./draft_push"
    } else {
        path.unwrap()
    };
    let aliasing = format!("!git push $1 $2 && {}", path);

    let _ = Command::new("git")
        .arg("config")
        .arg("--global")
        .arg("alias.xpush")
        .arg(aliasing)
        .output()
        .expect("Could not configure xpush as global git alias ?");
    if does_alias_exist() {
        println!("Alias successfully set");
    }
}

/// check if xpush alias exists already
/// git config --list | grep alias
pub fn does_alias_exist() -> bool {
    let output = Command::new("git")
        .arg("config")
        .arg("--list")
        .output()
        .expect("Could not configure xpush as global git alias ?");
    let msg = String::from_utf8_lossy(&output.stdout);
    msg.contains("alias.xpush")
}

pub fn write_token(data: &str) {
    if !Path::new(&get_cfg_home()).exists() {
        fs::create_dir(get_cfg_home()).unwrap();
    }
    let f = File::create(format!("{}/tk.txt", get_cfg_home())).expect("Unable to create file");
    let mut f = BufWriter::new(f);
    f.write_all(data.as_bytes()).expect("Unable to write data");
}

pub fn get_token() -> String {
    let file_path = format!("{}/{}", get_cfg_home(), "tk.txt");
    let mut token = String::new();
    let mut f = File::open(file_path).expect("Unable to open file");
    f.read_to_string(&mut token).expect("Unable to read string");
    //  println!("TOKEN: {}", token);
    if token.ends_with('\n') {
        token.pop();
    }
    token
}

pub fn get_cfg_home() -> String {
    let config_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME").map(|home| format!("{}/.config/draft_push", home)));
    config_home.unwrap()
}

#[tokio::main]
pub async fn get() -> Result<()> {
    let remote = &get_upstream();

    let (org_name, project_name, repo_name) = get_details(remote.into());
    let branch = get_current_branch();
    println!(
        "Repo: {:?} Project: {:?} Org: {:?}",
        repo_name, project_name, org_name
    );

    let token = get_token();
    let url = get_upstream();
    let client = get_client(&org_name, token, url);
    let options = PullListOptions::builder()
        .source_ref_name(format!("refs/heads/{}", branch))
        .build();

    // Only one pull request per branch, even though this is an array, there should be only 1 pull requst or none
    let pr = client
        .repo(&project_name, &repo_name)
        .pulls()
        .list(options)
        .await?;

    if pr.count > 0 {
        let pull = &pr.value[0];
        let pull_id = pull.pull_request_id;
        let is_draft = pull.is_draft;
        println!("Got pull request: id={} draft={}", pull_id, is_draft);

        // If it's already draft, publish right away
        if !is_draft {
            let _ = update_draft(&client, pull_id, true, remote.into()).await?;
        }
        let _ = update_draft(&client, pull_id, false, remote.into()).await?;
    } else {
        println!("No pull request found for {:?}", branch);
    }

    println!("Done");
    Ok(())
}

pub async fn update_draft(
    client: &AzureClient,
    pull_id: u64,
    is_draft: bool,
    remote: String,
) -> Result<PullRequestResponse> {
    let (_, project_name, repo_name) = get_details(remote);
    let draft_options = PullUpdateOptions::builder().draft(is_draft).build();
    let response = client
        .repo(project_name, repo_name)
        .pull(pull_id)
        .update(&draft_options)
        .await;
    response
}

// git config --get remote.origin.url
/// used to get project name / repo name / orgname  from the git remote url
pub fn get_upstream() -> String {
    let output = Command::new("git")
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()
        .expect("No remote url found ?");
    let msg = String::from_utf8_lossy(&output.stdout);
    msg.into()
}

// git rev-parse --abbrev-ref HEAD
pub fn get_current_branch() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .expect("No current branch ?");
    let msg = String::from_utf8_lossy(&output.stdout);
    let mut branch: String = msg.into();
    if branch.ends_with('\n') {
        branch.pop();
    }
    branch
}

// Get base url from the upstream url
pub fn get_base_url(url: String) -> String {
    let mut base_url: String = url;

    // replace ssh
    if base_url.starts_with("ssh") {
        base_url = base_url.replace("ssh", "https");
    }
    // start looping after https://, it's a base url if next character is : or /
    for x in 8..base_url.len() {
        let character = base_url.chars().nth(x).unwrap();
        if character == ':' || character == '/' {
            let (first, _) = base_url.split_at(x);
            base_url = first.to_string();
            break;
        }
    }
    base_url
}

// Returns the organization, project and repository we are currently in
pub fn get_details(remote: String) -> (String, String, String) {
    let project_index = 4;
    let org_index = 3;
    let list: Vec<&str> = remote.split("/").collect();
    let repo_name: &str = list.last().expect("Could not find repo name");
    let project_name: &str = list[project_index];
    let org_name: &str = list[org_index];

    let mut repo: String = repo_name.into();
    // sanitize repo string
    // this has to be done only for repo because of trailing new lines, and repo is always last
    if repo.ends_with("\n") {
        repo.pop();
    }

    (org_name.into(), project_name.into(), repo)
}

/// Returns a simple client to the other examples
pub fn get_client(org: &str, token: String, upstream: String) -> AzureClient {
    let agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let creds = Credentials::Basic(token);

    let mut azure = AzureClient::new(agent, org, creds).unwrap();

    // Client defaults to this anyway
    azure.set_api_version(ApiVersion::V5_1);

    // Set base host for the client based on the git remote upstream
    // if not set, it defaults to dev.azure.com
    let base_url = get_base_url(upstream);
    azure.set_host(base_url);

    azure
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{get_base_url, get_client, get_details, update_draft, PullListOptions};
    use dotenv;
    #[test]
    fn parse_https_base_url() {
        let url = "https://test.draftpush.com/Company/MyProject/_git/Project.Rust.App".into();
        let base = get_base_url(url);
        assert_eq!(base, "https://test.draftpush.com");
    }
    #[test]
    fn parse_ssh_base_url() {
        let url = "ssh://test.draftpush.com/Company/MyProject/_git/Project.Rust.App".into();
        let base = get_base_url(url);
        assert_eq!(base, "https://test.draftpush.com");
    }
    #[test]
    fn parse_ssh_port_base_url() {
        let url = "ssh://test.draftpush.com:44/Company/MyProject/_git/Project.Rust.App".into();
        let base = get_base_url(url);
        assert_eq!(base, "https://test.draftpush.com");
    }

    #[test]
    fn parse_https_user_base_url() {
        let url =
            "ssh://bentestingacc@test.draftpush.com:44/Company/MyProject/_git/Project.Rust.App"
                .into();
        let base = get_base_url(url);
        assert_eq!(base, "https://bentestingacc@test.draftpush.com");
    }

    #[test]
    fn parse_details() {
        let url = "https://test.draftpush.com/Company/MyProject/_git/Project.Rust.App".into();
        let (org_name, project_name, repo_name) = get_details(url);

        assert_eq!(org_name, "Company");
        assert_eq!(project_name, "MyProject");
        assert_eq!(repo_name, "Project.Rust.App");
    }

    #[test]
    fn parse_details_user() {
        let url =
            "https://bentestingacc@dev.azure.com/Company/MyProject/_git/Project.Rust.App".into();
        let (org_name, project_name, repo_name) = get_details(url);

        assert_eq!(org_name, "Company");
        assert_eq!(project_name, "MyProject");
        assert_eq!(repo_name, "Project.Rust.App");
    }

    #[test]
    fn parse_details_newline() {
        let url = "https://dev.azure.com/Company/MyProject/_git/Project.Rust.App\n".into();
        let (org_name, project_name, repo_name) = get_details(url);

        assert_eq!(org_name, "Company");
        assert_eq!(project_name, "MyProject");
        assert_eq!(repo_name, "Project.Rust.App");
    }

    #[tokio::main]
    #[test]
    async fn get_test_pr() {
        fn check_updated_draft(
            result: Result<PullRequestResponse>,
            pull_id: i64,
            should_be_draft: bool,
        ) {
            match result {
                Ok(res) => {
                    println!("Got pull request: res={:?}", res);
                    assert_eq!(res.pull_request_id, pull_id);
                    assert_eq!(res.is_draft, should_be_draft);
                }
                Err(e) => {
                    panic!("Failed to update api {:?}", e);
                }
            }
        }
        let remote = "https://dev.azure.com/bentestingacc/Azure-Testing/_git/Test\n";
        let (org, project, repo) = get_details(remote.into());
        let token = dotenv::var("AUTH_KEY").unwrap();
        println!("org={:?}, project={:?}, repo={:?}", org, project, repo);

        let client = get_client(&org, token, remote.into());

        let branch = "draft_push_ci";
        let options = PullListOptions::builder()
            .source_ref_name(format!("refs/heads/{}", branch))
            .build();

        let test = client.repo(&project, &repo).pulls().list(options).await;
        match test {
            Ok(result) => {
                if result.count > 0 {
                    let pull = &result.value[0];
                    let pull_id = pull.pull_request_id;
                    let is_draft = pull.is_draft;
                    println!("Got pull request: id={} draft={}", pull_id, is_draft);
                    if !is_draft {
                        let future = update_draft(&client, pull_id, true, remote.into()).await;
                        check_updated_draft(future, pull_id as i64, true);
                    }
                    let future = update_draft(&client, pull_id, false, remote.into()).await;
                    check_updated_draft(future, pull_id as i64, false);
                } else {
                    panic!("Pull request not found {:?}", result.count);
                }
            }
            Err(e) => {
                panic!("Request failed {:?}", e);
            }
        }
    }
}
