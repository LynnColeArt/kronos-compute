# Organization-wide README audit: Kronos Compute

Owner: Sam (product owner); implementation by Codex under the organization README audit.
Branch: `codex/readme-evidence-pointer`.
Baseline: `d93caef6cdba2134291f1a587be1707feae6bde1`.
Scope: `README.md` and this mission log.

History: [README orientation and evidence refresh](2026-10-16-readme-orientation.md)
and [repository guidance](../../AGENTS.md).

## User Experience Findings

- The README described its macOS validation as “This README review” without a
  date or a stable evidence link. Readers had to infer which review was meant
  and whether the toolchain and qualification limits were still current.
- The linked setup guide already carries a dated macOS validation record and
  explicitly says that the declared Rust minimum, Linux runtime, and GPU
  execution were not qualified.

## Engineering Decisions

- Point the README at the dated setup guide instead of repeating a transient
  reviewer-relative claim. Keep the untested minimum-version and Linux-runtime
  boundaries explicit.
- Make no source, package-metadata, hardware-qualification, or release changes.
  The current GitHub release and package-repository mismatch remain as recorded
  in the README.

## Known Limitations

- The declared Rust 1.70 minimum version and Linux runtime behavior remain
  untested. The recorded macOS checks do not dispatch GPU work.
- The package metadata still names the historical `LynnColeArt` repository;
  ownership and registry alignment remain unresolved before a future release.
