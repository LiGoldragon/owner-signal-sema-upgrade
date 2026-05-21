# owner-signal-sema-upgrade Architecture

This contract is the owner-only policy surface for `sema-upgrade`. It
does not run migrations. It lets the engine owner register migration
modules and decide which component/version transitions are allowed.

## Constraints

- Owner policy uses a separate signal contract from ordinary upgrade
  attempts.
- Owner requests may register or change migration policy; ordinary
  requests may only inspect, attempt, or report.
- The contract reuses `signal-sema-upgrade` version and migration
  identity records so the two sockets cannot drift on what a migration
  names.
- The prototype does not encode file permissions. Socket ownership is a
  daemon/deployment concern.
