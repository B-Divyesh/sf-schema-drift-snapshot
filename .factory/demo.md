# Schema Drift Snapshot demo sandbox

## CLI demo

Run `sds demo`. The command uses the two versioned PostgreSQL snapshots in
`examples/fixtures/` that are embedded in the binary at compile time. It
creates a unique directory under the operating system's temporary directory,
writes both snapshots and `drift-review.md`, then prints that path. The report
contains five classified differences: four high risk and one medium risk.

Use `sds demo --output <directory>` for a deterministic test location. Remove
that directory to reset. The command opens no database connection and changes
no user configuration.

## Browser demo

Open `/demo/` or
<https://schema-drift-snapshot.sociobot.in/demo/>. The page starts with a
four-change sample already classified. “Reset demo” restores the in-memory
fixtures. “Start for real” returns to the CLI installation section.

The browser sandbox stores no sample edits. Its effective storage namespace is
memory-only `demo:` state, separate from the site's `sb_license:*` localStorage
keys. Demo mode never reads or writes those real-mode keys.
