use crate::{
    config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository, ExecCommand,
};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use jira::{types::IssueKey, JiraAPIClient};

#[derive(Args, Debug)]
pub struct Transition {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Prompt for filter to use a default_query
    #[arg(short = 'f', long = "filter")]
    #[cfg(feature = "cloud")]
    use_filter: bool,
}

impl ExecCommand for Transition {
    fn exec(self, cfg: &Config) -> Result<String> {
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

        let transitions = client.get_transitions(&issue_key)?;
        let transition = if transitions.len() == 1 && cfg.one_transition_auto_move.unwrap_or(false)
        {
            transitions[0].clone()
        } else {
            Select::new("Move to:", transitions)
                .prompt()
                .wrap_err("No transition selected")?
        };

        client.post_transition(&issue_key, transition)?;
        Ok(String::default())
    }
}