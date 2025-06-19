use crate::command::GlobalArgs;
use crate::state::profiles::Profiles;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use eye_config::persistable_state::PersistableState;
use itertools::Itertools;

pub struct ProfileUseCommand;
impl ProfileUseCommand {
    pub async fn handle(self, _global: GlobalArgs, name: Option<String>) -> eyre::Result<()> {
        let mut profiles = Profiles::load().await?;
        match name {
            Some(name) => {
                let profiles_with_matching_name = profiles
                    .profiles
                    .iter()
                    .filter(|p| p.name == name)
                    .collect_vec();
                match profiles_with_matching_name.as_slice() {
                    [] => eyre::bail!("Profile '{}' does not exist", name),
                    [single] => {
                        // If there's exactly one profile with the given name, set it as active
                        profiles.active_profile = Some(single.name.clone());
                        profiles.save().await?;
                        println!("Active profile set to: {}", single.name);
                    }
                    _ => eyre::bail!(
                        "Multiple profiles found with the name '{}'. You will have to fix this manually at {}",
                        name,
                        Profiles::key().await?.file_path()?.display()
                    ),
                }
            }
            None => {
                let choices = profiles
                    .profiles
                    .iter()
                    .map(|profile| Choice {
                        key: profile.name.clone(),
                        value: profile,
                    })
                    .collect_vec();
                match choices.as_slice() {
                    [] => eyre::bail!("No profiles available. Please create one first."),
                    [single] => {
                        // If there's only one profile, set it as active
                        profiles.active_profile = Some(single.key.clone());
                        profiles.save().await?;
                        println!("Active profile set to: {}", single.key);
                    }
                    _ => {
                        // Prompt user to select a profile
                        let selected = cloud_terrastodon_user_input::pick(FzfArgs {
                            choices,
                            header: Some("Select a profile to set as active:".to_string()),
                            ..Default::default()
                        })?;
                        profiles.active_profile = Some(selected.name.clone());
                        profiles.save().await?;
                        println!("Active profile set to: {}", selected.name);
                    }
                }
            }
        }
        Ok(())
    }
}
