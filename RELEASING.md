# Releasing USB Camera

## Versioning

The project follows Semantic Versioning. Update the package version in `Cargo.toml`, update version-sensitive tests, regenerate `Cargo.lock` when required, and move the relevant entries from `Unreleased` into a dated section in `CHANGELOG.md`.

The first public release should use a new version after `1.0.4`; historical private tags must not be reused or moved.

## Release checks

Release from a clean, reviewed commit using Rust 1.97.1 or newer:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo audit --ignore RUSTSEC-2026-0041
cargo package --locked
```

Complete and record the applicable hardware checks, including continuous capture, all supported output formats, shutdown/reopen behavior, and the failure modes affected by the release.

USB Camera 2.1.0 resolves the published `forge_msgs 2.0.0` and
`forgelab_common 2.1.0` crates from crates.io. Keep `Cargo.lock` on registry
sources and require successful `cargo package --locked` verification before
release. This application is distributed through source tags and GitHub binary
assets, not crates.io.

The Dora 1.0.1 lock currently resolves `lz4_flex 0.10.0`, which triggers
`RUSTSEC-2026-0041`. The affected Zenoh block-decompression path is not compiled
in this build because transport compression is disabled, so the audit uses a
targeted exception until Dora/Zenoh updates the dependency.

## Source release

Before creating a tag:

1. Confirm `Cargo.toml`, `Cargo.lock`, `README.md`, and `CHANGELOG.md` agree on the version and requirements.
2. Confirm the package archive contains `LICENSE` and no private files or generated artifacts.
3. Confirm the repository working tree is clean and CI passes.
4. Create an immutable annotated tag named `v<version>` at the validated commit.

The package is marked `publish = false` and must not be uploaded to crates.io.
Do not store repository tokens in this repository. Published public release tags
must never be replaced.

## Binary release

The normative binary release path is to trigger the **Build Ubuntu 20.04 binary** workflow (`.github/workflows/build-ubuntu20.yml`) manually through GitHub Actions. Supply all three workflow inputs:

- `ref`: the immutable release tag to check out, exactly `v<version>`.
- `version`: the package version without the `v` prefix. It must match `ref` and the exact `usb_camera <version>` output embedded in the binary.
- `upload_to_release`: whether to attach the verified archive and checksum to GitHub Releases. It defaults to `false`, so a normal run only stores a workflow artifact. Set it to `true` only after creating an existing draft release whose tag exactly matches `ref`; the workflow does not create the release and refuses to replace an existing asset with different content.

A single workflow run builds both supported architectures with Rust 1.97.1. The matrix uses the native `ubuntu-latest` runner with target `x86_64-unknown-linux-gnu` for x86_64 and the native `ubuntu-24.04-arm` runner with target `aarch64-unknown-linux-gnu` for ARM64. Each native runner executes the build in the matching architecture of the Ubuntu 20.04 container, preserving the Ubuntu 20.04/glibc 2.31 baseline for both binaries. The workflow applies path remapping through environment variables such as `GITHUB_WORKSPACE`, strips symbols, and produces dynamically linked GNU/glibc binaries whose maximum required glibc symbol version must not exceed 2.31. The build commands in the workflow are implementation details for those containers and are not a supported copy-and-paste local release procedure.

For each matrix architecture, the workflow verifies the exact embedded version, confirms that `file` reports a 64-bit ELF binary, checks the expected machine (`Advanced Micro Devices X86-64` or `AArch64`) with `readelf -h`, uses `ldd` to ensure that all dynamic libraries resolve, checks the maximum required glibc symbol version with `objdump -T`, and scans for private build markers. After packaging, it verifies the SHA-256 checksum and extracts the final archive to run an additional `--version` smoke test.

Do not reuse files from the local `dist/` directory. The `usb_camera_test_sink` binary is a development and Dora example utility and must not be included in public binary archives.

The minimal public binary archives are `usb_camera-v<version>-ubuntu20.04-x86_64.tar.gz` and `usb_camera-v<version>-ubuntu20.04-arm64.tar.gz`. Each contains only the stripped `usb_camera` executable and has a matching `.tar.gz.sha256` file. Every workflow run uploads the archive and checksum from each matrix job as architecture-specific GitHub Actions artifacts retained for 30 days; enabling `upload_to_release` additionally attaches both architecture sets to the matching draft release. Project documentation and licensing remain available in the repository and GitHub-generated source archives.

Downloading each final archive and testing it on a separate, clean Ubuntu 20.04 runtime of the matching architecture is an additional manual release step: validate the x86_64 archive on clean x86_64 and the arm64 archive on clean ARM64/aarch64. The current workflow's build-container checks and extracted-archive smoke tests do not perform or replace either clean-runtime validation. Complete and record both manual tests before publishing the draft release.
