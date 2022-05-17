# Paste

## Database

Using sqlite via `sqlx` for the database. Migrations are automatically run as
part of server start. See
[rocket docs](https://rocket.rs/v0.5-rc/guide/state/#databases) for details of
how this is configured.

`sqlx-cli` is used locally for development and testing.
