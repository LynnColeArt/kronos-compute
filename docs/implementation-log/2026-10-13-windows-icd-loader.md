# Windows ICD loader compilation — 2026-10-13 UTC

## User Experience Findings

- The required Windows build check for a documentation-only PR failed before
  tests. The loader imported `std::os::unix::ffi::OsStrExt` on Windows, where
  that module does not exist. The red check gave the policy PR no usable
  review signal despite the policy edit being unrelated.

## Engineering Decisions

- Limit the Unix path conversion trait to Unix targets and the `CStr` import
  to non-Windows targets. Windows continues to load the canonicalized path
  through `libloading`; Unix continues to pass a NUL-terminated path to
  `dlopen`. The change does not alter ICD trust policy or lookup behavior.

## Known Limitations

- Local validation runs on macOS. The Windows `build-and-unit-tests` check
  passed on PR #11 (run 38340930943). The blocked policy PR must be rerun
  against the repaired main branch. The separate legacy CI workflow remains
  red on repository-wide formatting and Vulkan SDK installation; those
  failures predate this import fix.
