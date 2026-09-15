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
cargo audit
cargo package --locked
cargo publish --locked --dry-run
```

Complete and record the applicable hardware checks, including continuous capture, all supported output formats, shutdown/reopen behavior, and the failure modes affected by the release.

## Source release

Before creating a tag:

1. Confirm `Cargo.toml`, `Cargo.lock`, `README.md`, and `CHANGELOG.md` agree on the version and requirements.
2. Confirm the package archive contains `LICENSE` and no private files or generated artifacts.
3. Confirm the repository working tree is clean and CI passes.
4. Create an immutable annotated tag named `v<version>` at the validated commit.

Do not store crates.io or repository tokens in this repository. Published crates.io versions and public release tags must never be replaced.

## Binary release

The existing portable x86_64 release path builds the user-facing binary as a static PIE linked against musl. Install `musl-tools`, add the Rust target, and build with path remapping and symbol stripping:

```bash
rustup target add --toolchain 1.97.1 x86_64-unknown-linux-musl
RUSTFLAGS="--remap-path-prefix=${HOME}=/build -C link-self-contained=yes" \
CARGO_PROFILE_RELEASE_STRIP=symbols \
cargo +1.97.1 build --locked --release --bin usb_camera \
  --target x86_64-unknown-linux-musl
```

Verify that `file` reports `static-pie linked`, `ldd` reports `statically linked`, and `readelf -l` does not report an `INTERP` segment. The minimal archive is named `usb_camera-v<version>-linux-x86_64-musl.tar.gz` and contains only the stripped `usb_camera` executable.

### Ubuntu 20.04 x86_64 and ARM64 artifacts

Trigger the **Build Ubuntu 20.04 binary** workflow (`.github/workflows/build-ubuntu20.yml`) manually through GitHub Actions with these inputs:

- `ref`: the immutable release tag to check out, exactly `v<version>`.
- `version`: the package version without the `v` prefix. It must match `ref` and the version embedded in the binary.

A single workflow run builds both architectures with Rust 1.97.1. The matrix uses the native `ubuntu-latest` runner with target `x86_64-unknown-linux-gnu` and the native `ubuntu-24.04-arm` runner with target `aarch64-unknown-linux-gnu`. Both run inside matching-architecture Ubuntu 20.04 containers, producing dynamically linked GNU/glibc binaries whose maximum required glibc symbol version must not exceed 2.31.

For each architecture, the workflow verifies the embedded version, 64-bit ELF format, expected `readelf` machine, dynamic-library resolution, GLIBC symbol ceiling, private build markers, archive contents, and SHA-256 checksum. It uploads these architecture-specific Actions artifacts for 30 days:

- `usb_camera-v<version>-ubuntu20.04-x86_64.tar.gz`
- `usb_camera-v<version>-ubuntu20.04-arm64.tar.gz`

Each archive has a matching `.tar.gz.sha256` file and contains only the stripped `usb_camera` executable. The `usb_camera_test_sink` development utility must not be included. Do not reuse files from the local `dist/` directory.

Before publishing either workflow artifact, download it and test it on a separate, clean Ubuntu 20.04 runtime of the matching architecture. Validate x86_64 on clean x86_64 and arm64 on clean ARM64/aarch64. Scan final binaries for private paths and internal URLs, verify their SHA-256 digests, and do not replace published assets with differing files.
