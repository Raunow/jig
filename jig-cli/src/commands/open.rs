use crate::interactivity::issue_from_branch_or_prompt;
use crate::{config::Config, repo::Repository, ExecCommand};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use jira::types::IssueKey;
use jira::JiraAPIClient;
use std::env;
use std::process::Command;

#[derive(Args, Debug)]
pub struct Open {
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Prompt for filter to use a default_query
    #[arg(short = 'f', long = "filter")]
    #[cfg(feature = "cloud")]
    use_filter: bool,
}

impl ExecCommand for Open {
    fn exec(self, cfg: &Config) -> Result<String> {
        let browser = env::var("BROWSER").wrap_err("Failed to open, missing 'BROWSER' env var")?;
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name(),
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            #[cfg(feature = "cloud")]
            let issue_key = issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)?.key;
            #[cfg(not(feature = "cloud"))]
            let issue_key = issue_from_branch_or_prompt(&client, cfg, head)?.key;
            issue_key
        };

        let result = Command::new(browser.clone())
            .args([format!("{}/browse/{}", client.url, issue_key)])
            .spawn();
        match result {
            Ok(_) => Ok(String::default()),
            Err(e) => Err(e).wrap_err(format!("Failed to open {} using {}", issue_key, browser)),
        }
    }
}