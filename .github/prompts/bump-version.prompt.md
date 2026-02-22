---
agent: agent
description: Bump the version of this project, commit the change and create a new git tag.
argument-hint: '[major|minor|patch]'
model: Auto (copilot)
---
To bump the version of this project, follow these steps:

1. Determine the version number from the workspace’s [Cargo.toml](../../Cargo.toml).
2. Update the version number in the [Cargo.toml](../../Cargo.toml) file to the new version.
3. Run `cargo check` to update the `Cargo.lock` file with the new version.
4. Commit the changes to the `Cargo.toml` and `Cargo.lock` files with a commit message like "Bump version to X.Y.Z".
5. Create a new git tag with the new version number using `git tag -m vX.Y.Z vX.Y.Z` (replace X.Y.Z with the new version number).
6. Push the commit and the new tag to the remote repository using `git push origin master --tags`.
