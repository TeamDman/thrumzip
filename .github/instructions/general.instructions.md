---
applyTo: '**'
---

Each new type should be placed in a new `{type_name}.rs` file.

Argument parsing should be done with `clap` using the `derive` feature.
There should be a `Command` struct with a subcommand enum.
Each subcommand enum variant should be a tuple struct with a single field of a `{CommandName}` struct or enum to contain the logic of the subcommand.
There should be a `GlobalArgs` struct to contain the global arguments, including a `--debug` and `--non-interactive` flag.
The command struct should have a `handle` method that matches on owned `self` which matches on the subcommand and calls the `handle` method of the subcommand struct, passing the global arguments.
The `Command` struct should use the `version` derive feature of `clap` to automatically generate the version information.

There should be a `init_tracing` function that takes a `Level` argument and initializes tracing subscriber with the given level.
In release builds, the logs should contain the timestamp, and the log level.
Only in debug builds, the logs should contain the file and line number and not the timestamp.

`eyre` should be used for all error handling.
`color-eyre` should be used to enable colored error messages.

The main fn should be approximately as follows:

1. Init color eyre
2. Parse the arguments
3. Init tracing with the log level determined from the global args
4. Call the `handle` method of the `Command` struct with the parsed arguments


All functions which can error should return `eyre::Result<{}>`.
The `eyre::Result` type should always be used inline and not imported at the top of the file.

`bail!` from `eyre` should be used for short-circuiting operations.

`wrap_err` from `eyre` should be used to wrap errors with context.

Dedicated types should be created for domain objects.
The `holda` crate can be used to easily create these types, since the `#[derive(holda::Holda)]` and `#[derive(holda::StringHolda)]` derive macros automatically implement most common traits.
`#[holda(NoHash)]` and similar can be used to suppress the automatic trait implementation when necessary, enabling you to provide your own implementation when needed.

`tokio` should be used for async operations.
Maximize parallel operations by using join sets and other tokio utilities.

If interaction is needed from the user, use the `cloud_terrastodon_user_input` crate.
The `pick` and `pick_many` functions are useful for having the user pick from a list of options.
The `Choice<T>` passed as `choices: Vec<Choice<{T}>>` in the `FzfArgs {, ..}` parameter to these pick methods has two fields: `key` and `value`.
The `key` is the value that will be displayed to user, it is the string that uniquely identifies the choice.
The `value` is the inner value that is desired after the user has made their selection.
The `pick` and `pick_many` methods will return the choices provided, so you can retrieve the key and value as needed.

Strong typing should be used everywhere.
If something is like a path, use a `PathBuf`.
If a type is holding a `PathBuf`, it should be made a domain object like `PathToZipFile` or `PathInsideZipFile` as appropriate.

Prefer `Local` from `chrono` for date and time handling of information that is not explicitly Utc.

We are designing specifically for the Windows platform.

When writing files to disk while we have additional metadata, such as date created or modified, ensure that the metadata is set correctly.
When writing files, use `tokio::fs` for async file operations.

Include `debug!` and `info!` logs in the code to provide useful information about the operations being performed.


Use the `eye_config` crate for storing persistent information such as preferences and user-configured properties.

Files should be small, less than 500 lines.
Break things up, create individual files to hold utility methods to ensure that logic is easily digestable and self-documenting via the use of domain object types.