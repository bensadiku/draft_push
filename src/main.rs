use azure_rs::pull_requests::{PullListOptions, PullUpdateOptions};
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
    init_clap();
    println!("Hello, world!");
    println!("get_upstream: {:?}", get_upstream());
    println!("current_branch: {:?}", get_current_branch());
    println!("get_last_commit {:?}", get_last_commit());

    println!("getting now");
    let _ = get();

    Ok(())
}

/// cargo run -- --config <TokenHere>
/// ./draft_push --config 12
pub fn init_clap() {
    let matches = App::new("Draft push")
        .version("1.0")
        .author("Behxhet S. <bensadiku65@gmail.com>")
        .about("Auto draft/publish pr after making changes")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .required(false)
                .help("Sets a token, get the token from azure")
                .takes_value(true),
        )
        .get_matches();
    // Gets a value for config if supplied by user, or defaults to "null"
    // in which case, check if we have a token locally
    let config = matches.value_of("config").unwrap_or("null");
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
    }
    //println!("Value for config: {}", config);
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
    let remote = get_upstream();

    // let remote = "https://tfs.company.com:22/Company/MyProject/_git/Project.Rust.App";

    let list: Vec<&str> = remote.split("/").collect();
    let repo_name: &str = list.last().expect("Could not find repo name");
    let project_name: &str = list[4];
    let org_name: &str = list[3];
    let branch = get_current_branch();
    println!(
        "Repo: {:?} Project: {:?} Org: {:?}",
        repo_name, project_name, org_name
    );

    let client = get_client(org_name);
    let options = PullListOptions::builder()
        .source_ref_name(format!("refs/heads/{}", branch))
        .build();
    let pr = client
        .repo(project_name, repo_name)
        .pulls()
        .list(options)
        .await?;
    println!("Got pull request");

    let draft_options = PullUpdateOptions::builder().draft(true).build();

    let _ = client
        .repo(project_name, repo_name)
        .pull(pr.value[0].pull_request_id)
        .update(&draft_options)
        .await;

    println!("Set pull request to draft");

    let draft_options = PullUpdateOptions::builder().draft(false).build();
    let _ = client
        .repo(project_name, repo_name)
        .pull(pr.value[0].pull_request_id)
        .update(&draft_options)
        .await;
    println!("Removed pull request from draft");

    Ok(())
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

// git rev-parse HEAD
pub fn get_last_commit() -> String {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
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
        .expect("No remote url found ?");
    let msg = String::from_utf8_lossy(&output.stdout);
    let mut branch: String = msg.into();
    if branch.ends_with('\n') {
        branch.pop();
    }
    branch
}

/// Returns a simple client to the other examples
pub fn get_client(org: &str) -> AzureClient {
    let agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let token = get_token();
    let creds = Credentials::Basic(token);

    let mut azure = AzureClient::new(agent, org, creds).unwrap();

    // Client defaults to this anyway
    azure.set_api_version(ApiVersion::V5_1);

    azure
}
