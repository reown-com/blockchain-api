# Migrations

This folder contains SQL migration scripts and they are automatically run on start-up.

## New Migration

To create new migration file sqlx-cli must be installed:

```
cargo install sqlx-cli
```

Create a new migration with the `name`:

```
sqlx migrate add <name>
```
