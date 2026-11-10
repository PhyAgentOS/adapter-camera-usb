[English](README.md) | [中文](README_zh.md)

# USB camera

Linux V4L2/UVC camera capture with raw RGB, JPEG and PNG Dora outputs.

## Migration status

Target node version: **2.1.2**. These changes are candidates pending the platform,
hardware and sustained-stream acceptance gates below. A version declaration does not
mean a stable release has been accepted.

usb_camera_test_sink validates every frame, fails on malformed or insufficient input, and supports min_frames and delay_ms for slow-consumer tests.

## Runtime and platform

- First validation combination: Dora CLI/coordinator/daemon/node APIs 1.0.1,
  Forge Msgs 2.0.0 and Common 2.1.0; use only the families required by this node.
- Forge wire schemas remain `forge_msgs.v1`. Dora 0.x and 1.x cannot share a runtime.
- Use Python 3.12 for Python builds. Camera-only Python packages may also declare
  Python 3.11 support; test that interpreter separately before claiming it.
- Rust packages enable `arrow-v59`; the committed lock uses Arrow 59.3.0.
- Hardware camera/robot artifacts target Linux. macOS development tests do not
  qualify Linux binaries. GPU/rendering and vendor SDK prerequisites still apply.
- Keep a recoverable `stable/1.x` runtime and artifact combination.

## Setup and commands

Run from the repository root:

```sh
cargo build --locked
usb_camera --config config/sensor.example.yaml
```

Use `usb_camera --help` (or `uv run --frozen usb_camera --help`) for supported CLI options.
The example configuration is `config/sensor.example.yaml`; choose the actual device and supported
stream profiles before starting. This command requires an appropriate Dora dataflow.

## Message contract

The configured output_id carries Forge Image or CompressedImage. Capture timestamps are preserved as user metadata.

Keep bounded queues for high-rate image consumers and document whether old images
are dropped or producers experience backpressure. Preserve image dimensions, dtype,
encoding, units and capture metadata across the node boundary.

## Tests and build

```sh
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --release --locked
```

Use the committed `Cargo.lock` and frozen/locked installs. Python vendor SDK tests require
the supported SDK platform. For Linux headless MuJoCo rendering, install the required
EGL or OSMesa libraries and select the appropriate `MUJOCO_GL` backend.

Validate every delivered example or rendered template with:

```sh
dora validate path/to/dataflow.yaml
```

Schema validation is followed by a real start, message exchange and stop test.
Keep the existing detailed configuration and device setup reference in
[the Chinese guide](README_zh.md) and the repository's `config/`, `examples/` and
`scripts/` directories where present.

## Acceptance and release

1. Build candidate archives from the selected commit/tag and record actual embedded
   dependency versions, platform, archive SHA256 and unpacked executable SHA256.
2. Install the final archive into a clean supported environment. Verify the CLI,
   message contract, resource cleanup and error exits.
3. For image producers, run real supported RGB/depth configurations for at least
   four continuous hours after warmup. Exercise slow/multiple consumers, stalled
   consumption, failures and repeated start/stop. Define throughput, latency and
   memory limits before testing, and retain time series rather than frame payloads.
4. Exercise the relevant device; synthetic streams do not qualify hardware capture.
   Demonstrate rollback to the complete old node/runtime combination.
5. Publish the exact accepted archive. Rebuilding invalidates that artifact's
   acceptance; repeat it before publication. Never overwrite published tags/assets.

## License

Apache-2.0. See `LICENSE` and any bundled third-party notices.
