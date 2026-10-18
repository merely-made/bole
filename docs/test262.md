# test262 conformance

Measured 2026-10-18 on bole `aef1134b` (genet) with `boa_tester` in release
mode, test262 `d86b2294` (the pin in `test262_config.toml`), the whole `test/`
tree, and the config's ignore list. Pass, fail and ignored are test counts from
the tester summary; the test262 commit is the checked-out one, not the ref the
tester prints in `latest.json`.

| Build | Total | Pass | Fail | Ignored | Conformance |
| :-- | --: | --: | --: | --: | --: |
| With Intl (default features) | 53,578 | 51,225 | 572 | 1,781 | 95.61% |
| Without Intl | 53,578 | 50,550 | 1,247 | 1,781 | 94.35% |

Per suite, with Intl (without Intl in brackets where it differs):

| Suite | Total | Pass | Ignored |
| :-- | --: | --: | --: |
| `built-ins` | 23,812 | 23,038 (23,036) | 763 |
| `language` | 23,724 | 23,087 | 582 |
| `intl402` | 3,357 | 2,624 (1,950) | 342 |
| `annexB` | 1,086 | 1,040 | 27 |
| `staging` | 1,483 | 1,320 (1,321) | 67 |
| `harness` | 116 | 116 | 0 |

Annex B is on in both builds (the `annex-b` feature is in the tester's
default set).

Reproduce from this directory (the first build clones test262 unless
`--test262-path` points at a checkout of the pinned commit):

```sh
# With Intl
cargo run --release --bin boa_tester -- run --test262-path <test262>
# Without Intl; -p is needed, a bare `--bin` build keeps Intl on
cargo run --release -p boa_tester --no-default-features \
  --features boa_engine/experimental,annex-b -- run --test262-path <test262>
```

## Ignored

Ignored tests are those whose `features` are in `test262_config.toml`, plus
the individual tests listed there under `tests`. Ignored features: 
`symbols-as-weakmap-keys`, `Intl.DisplayNames`, `Intl.RelativeTimeFormat`,
`Intl-enumeration`, `Intl.DurationFormat`, `regexp-duplicate-named-groups`,
`explicit-resource-management`, `joint-iteration`, `import-bytes`,
`Intl.Locale-info`, `legacy-regexp`, `import-defer`, `ShadowRealm`,
`decorators`, `uint8array-base64`, `source-phase-imports-module-source`,
`immutable-arraybuffer`, `iterator-includes`, `iterator-chunking`,
`Iterator.prototype.join`, `error-stack-accessor`, `caller`.
