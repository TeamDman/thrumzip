use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use cloud_terrastodon_user_input::pick_many;

fn main() -> eyre::Result<()> {
    let chosen = pick(FzfArgs {
        choices: (1..10)
            .map(|i| Choice {
                key: format!("Number {i}"),
                value: i,
            })
            .collect(),
        header: Some("Pick a number".to_string()),
        ..Default::default()
    })?;
    // the key is available if we want
    // Choice implements deref if we want to use the value without needing .value anywhere
    println!("You picked: {}", *chosen);

    let chosen = pick_many(FzfArgs {
        choices: (1..10)
            .map(|i| Choice {
                key: format!("Number {i}"),
                value: i,
            })
            .collect(),
        header: Some("Pick some numbers".to_string()),
        ..Default::default()
    })?;
    println!(
        "You picked: {:?}",
        chosen
            .into_iter()
            .map(|choice| choice.value)
            .collect::<Vec<_>>()
    );
    Ok(())
}
