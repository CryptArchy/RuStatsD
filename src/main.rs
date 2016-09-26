#[macro_use]
extern crate version;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate clap;
extern crate mio;
extern crate bytes;

use std::cell::Cell;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::process;
use clap::{Arg, App, AppSettings};

mod metrics;
mod frontends;
mod backends;
use frontends::*;
use backends::*;

struct UserConfiguration {
    owner: String,
    is_user: bool,
    repo: Option<String>,
    branch: Option<String>,
    sha: Option<String>,
}

struct Bucket {
    name: String,
    value: Cell<i64>,
    sampling_rate: Cell<f64>,
}

fn main() {
    log4rs::init_file("log4rs.toml", Default::default()).unwrap();
    let ver: version::Version = std::str::FromStr::from_str(version!()).unwrap();
    println!("RuStatsD v{}", ver);
    info!("RuStatsD v{}", ver);

    let mut buckets = HashMap::new();
    let bucket = buckets.entry("stats.counters.whatever").or_insert(0);
    *bucket += 1;

    // main_tcp();
    let host = "127.0.0.1";
    let port = "13265";
    let mut server = udp_server::UdpReader::new(&host, &port).unwrap();
    server.run();
}

    // env_check("INBOUND_ADDRESS", "HTTP endpoint for incoming StatsD messages");
    // env_check("OUTBOUND_ADDRESS", "HTTP endpoint for outgoing Graphite messages");

    // let matches = App::new("rustatsd")
    //     .setting(AppSettings::ColoredHelp)
    //     .setting(AppSettings::UnifiedHelpMessage)
    //     .setting(AppSettings::DeriveDisplayOrder)
    //     .version(version!())
    //     .arg(Arg::with_name("owner")
    //         .index(1)
    //         .value_name("OWNER")
    //         .help("Github repo owner")
    //         .takes_value(true)
    //         .required(true))
    //     .arg(Arg::with_name("user")
    //         .short("u")
    //         .long("user")
    //         .help("Use a Github user instead of an organization as owner")
    //         .takes_value(false))
    //     .arg(Arg::with_name("repo")
    //         .short("r")
    //         .long("repo")
    //         .value_name("REPO_NAME")
    //         .help("Github repo name")
    //         .takes_value(true))
    //     .arg(Arg::with_name("branch")
    //         .short("b")
    //         .long("branch")
    //         .value_name("BRANCH")
    //         .help("Repo branch to use. Defaults to `master`")
    //         .requires("repo")
    //         .takes_value(true))
    //     .arg(Arg::with_name("sha")
    //         .short("s")
    //         .long("sha")
    //         .value_name("SHA")
    //         .help("Search for a SHA. Partial SHA matches")
    //         .requires("repo")
    //         .takes_value(true))
    //     .arg(Arg::with_name("dry")
    //         .short("d")
    //         .long("dry")
    //         .help("Dry run. Don't post to Grafana")
    //         .requires("repo")
    //         .takes_value(false))
    //     .get_matches();

    // let state = UserConfiguration {
    //     owner: String::from(matches.value_of("owner").unwrap()),
    //     is_user: matches.is_present("user"),
    //     repo: upgrade_str_option(matches.value_of("repo")),
    //     branch: upgrade_str_option(matches.value_of("branch")),
    //     sha: upgrade_str_option(matches.value_of("sha")),
    // };

    // let broadcast_ip = match env::args().nth(1) {
    //     Some(host) => host,
    //     None => String::from("0.0.0.0"),
    // };
    // let host = "0.0.0.0";
    // let port = "6881";
    // let mut server = DogtownServer::new(&host, &port, &broadcast_ip);
    // server.run();

    // match state {
    // AppState { owner, is_user, repo: None, branch: None, sha: None } => {
    // let repos_url: String;
    // if is_user {
    // repos_url = user_repos_url(&owner);
    // } else {
    // repos_url = org_repos_url(&owner);
    // }
    // let response = http_get(&repos_url);
    // let repos: Vec<RepoResponse> = json::decode(&response).unwrap();
    // for repo in repos {
    // println!("{}/{}", owner, repo.name)
    // }
    // }
    // AppState { owner, is_user: _, repo: Some(repo), branch, sha: None } => {
    // let commits_url = repo_commits_url(&owner, &repo, branch);
    // let response = http_get(&commits_url);
    // let commits: Vec<RepoCommitResponse> = json::decode(&response).unwrap();
    // match commits.first() {
    // Some(commit) => handle_commit(&repo, &commit, matches.is_present("dry")),
    // None => (),
    // }
    // }
    // AppState { owner, is_user: _, repo: Some(repo), branch, sha: Some(sha) } => {
    // let commits_url = repo_commits_url(&owner, &repo, branch);
    // let response = http_get(&commits_url);
    // let commits: Vec<RepoCommitResponse> = json::decode(&response).unwrap();
    // match commits.into_iter().filter(|ref c| c.sha.starts_with(&sha)).next() {
    // Some(commit) => handle_commit(&repo, &commit, matches.is_present("dry")),
    // None => (),
    // }
    // }
    // _ => panic!("Unsupported option combination"),
    // };

fn env_check(env_var: &str, help: &str) {
    if let Err(_) = env::var(env_var) {
        println!("Missing environment variable: `{}` -- {}", env_var, help);
        process::exit(1);
    }
}

fn upgrade_str_option(o: Option<&str>) -> Option<String> {
    match o {
        Some(s) => Some(String::from(s)),
        None => None,
    }
}
