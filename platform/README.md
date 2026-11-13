# Platform builds

Current console execution and memory findings (2026-10-24):
[emulator validation](../reverse/console-emulator-validation.md) and
[ADR-0007](../reverse/adr/0007-verify-console-runtime-and-memory.md).
These supersede earlier build-only console acceptance statements below.

Latest action/transition follow-up: [port acceptance status](../reverse/port-status.md)
and [validation record](../reverse/evidence/action-transition-validation.json).
It passed 271 focused desktop tests (5 ignored), 179 host-audio library tests
(4 ignored), and 14 script tests. The seven native platform build/check scopes
remain distinct from device testing. Older validation counts below are historical.

All frontends run the same PAL VM and the MPEG/WMV decoders imported from
[siglus_rs](https://github.com/xmoezzz/siglus_rs). Game assets are external;
packages do not include a game. Keep the original game installation intact and
copy it to a writable device directory for gameplay and saves.

| Platform | Entry or build command | Output |
|---|---|---|
| macOS | `platform/scripts/package_macos_dmg.sh` | DMG with the native launcher |
| iOS | `platform/scripts/package_ios_altstore_ipa.sh` | IPA; Xcode and the signing requirements in the script apply |
| Android | `platform/scripts/package_android_apk.sh` | APK; Android SDK/NDK and JDK required |
| Linux | `cargo build --release -p pal-vm --bin sena` | `target/release/sena` |
| Windows | `platform/scripts/build_windows_msvc.cmd` | Windows executable; run in the MSVC environment |
| Web | `platform/scripts/build_wasm.sh` | WASM browser shell |
| Switch | `platform/switch/build_switch.sh` | `dist/switch/sena.nro` |
| PS Vita | `platform/vita/build_vita.sh` | `dist/vita/sena.vpk` |

See [Switch](switch/README.md) and [PS Vita](vita/README.md) for device paths and
controls. The existing release workflow now includes both console artifacts.

Android main/tag packages use a fixed release signing key and are verified
before upload. PR and feature-branch artifacts use debug signing while retaining
optimized Rust builds. Moving from an older debug-signed package to the fixed
release key requires backing up app data and uninstalling the old package once;
later releases can update the installed app with the same signing identity.

Local release APKs are unsigned by default. Set `ANDROID_KEYSTORE`,
`ANDROID_KEY_ALIAS`, `ANDROID_KEYSTORE_PASSWORD`, and `ANDROID_KEY_PASSWORD`, then
run `bash platform/android/sign.sh INPUT.apk OUTPUT.apk` to align, sign, and
verify one. CI uses repository secrets `ANDROID_KEY_BASE64` (the encoded
keystore) and the same alias/password names; a missing key stops publication.
Keep a separate backup of the keystore and passwords outside the repository.

## Continuous integration

[Build Artifacts](../.github/workflows/release.yml) builds Switch and Vita in
separate Linux x86-64 Docker jobs on pushes, main-branch pull requests, and manual
workflow dispatch. The existing host-audio library and C input checks remain
prerequisites. Jobs run directly in siglus_rs's published GHCR images, pinned
by immutable manifest digest. Switch uses its prepared `nightly-2026-01-20`;
Vita uses its prepared `nightly-2026-09-18`. Neither job rebuilds an SDK image.
Only the small CMake/Python/patch packaging helpers are installed at job startup.
Native build caches are scoped by SDK digest, toolchain, build scripts and
dependency locks. See [image identities and provenance](UPSTREAM.md).

Successful jobs upload `sena-switch` (`sena.nro` and `sena.nro.sha256`) and
`sena-vita` (`sena.vpk` and `sena.vpk.sha256`). Package checks reject missing or
truncated NRO headers, invalid Vita SELF/SFO entries and corrupt VPK CRCs.
The existing rolling pre-release and version-tag release jobs include both
packages and checksums. Failed native builds retain a separate build-log
artifact. Docker build records are excluded from release assets.

The CI maintenance check passed 252 existing host-audio library tests (5 ignored),
the existing C input check, workflow/actionlint validation, and verification of
the locally built NRO/VPK. Four stale test assumptions were updated for PAL-clock
fades, the group slider-end latch, restored dynamic strings, and the independent
v11 wire layout. No new regression tests or native ABI changes were added.
The first hosted run passed those checks and produced the Vita VPK; Switch
failed while our old Dockerfile requested the nonexistent `switch-deko3d` package.
The prepared-image correction removes that SDK rebuild. Its hosted result must
be checked separately from the preceding local validation.

## Core and host boundary

`pal-vm` defaults to `desktop` and `device-audio`: the current winit/wgpu window
host and CPAL audio. iOS, Android, macOS, Linux, and Windows retain that build
configuration and their existing C ABI. `cli` enables the resource utilities.

Console wrappers depend on `pal-vm` with `default-features = false` and
`features = ["host-audio"]`. This excludes winit, wgpu, softbuffer, and CPAL.
The host steps the VM and maps native input through the same letterbox
rectangle. Both consoles use the borrowed texture/draw scene ABI: Switch renders through
deko3d, and Vita uses vita2d/GXM precompiled programs for ordinary draws. Both
provide a software option; Vita requests CPU frames on demand for PAL raster. A native audio thread pulls signed 16-bit
stereo PCM at 48 kHz from Kira. The opaque sink owns a shared mixer reference,
not an engine pointer, so audio never aliases the mutable FFI host. Stop and join
the audio thread before destroying its sink; engine destruction silences any
surviving sink. A failed audio device drains PCM silently to keep playback waits
advancing. The no-feature build uses Kira's mock backend for core tooling.

Switch uses native ordinary/raster shaders compiled from the same WGSL as
the desktop shader sources; global raster prefix capture remains on the CPU.
Vita uses native quads for ordinary draws and the software compositor for PAL
raster effects. Its embedded vita2d shaders require no `libshacccg.suprx`.
That module is only needed by the referenced tool for compiling new or changed
Cg programs; it is not a prerequisite for GXM rendering. See the
[Vita follow-up](../reverse/vita-precompiled-rendering.md). Memory use, audio latency, suspend/resume, and long play
sessions need device testing; full-speed 1080p playback is not certified. MP4 is explicitly rejected; native playback support is not established. See
[native movie-format evidence](../reverse/movie-format-policy.md).

## Validation

On 2026-10-23 the local SDKs produced both final NRO and VPK packages, using
`nightly-2026-01-20` with `rust-src`. The Vita SFO requests extended memory
(`ATTRIBUTE2 = 12`) for its 320 MiB heap. The console control/letterbox test and
host PCM mixer tests passed in that earlier batch. The subsequent 2026-10-24
[console emulator run](../reverse/console-emulator-validation.md) supersedes its
build-only scope; no hardware run is claimed.

The Dockerfiles follow the SDK bootstrap used by siglus_rs. The Docker builds
and hosted GitHub Actions jobs have not been run locally; their external SDK
images and package repositories can change. Existing platform packaging and
device signing are separate from Rust compilation checks.

| Local validation | Result |
|---|---|
| Default VM library and focused effect/engine/movie/MPEG/operand/runtime/sprite/system/task integrations | 236 passed, 5 ignored |
| VM library with only `host-audio` | 150 passed, 4 ignored |
| WMV decoder library tests | 15 passed |
| MPEG decoding fixtures in `pal-vm/tests/mpeg2_movie.rs` | 3 passed, included in the default-feature scope above |
| iOS arm64, Android arm64, Windows x64 GNU, and WASM default-feature library checks | Passed after FX/scene ABI changes |
| Linux x64 core check with `host-audio` | Passed; desktop Linux packaging was not run on this Mac |
| macOS desktop executable build | Passed |
| Shared C cursor, touch capture, and letterbox tests | Passed with warnings treated as errors |
| Switch native shader/NRO build and Vita VPK/SFO inspection | Passed; four DKSH programs embedded; Vita `ATTRIBUTE2 = 12` |
| Actual Metal raster GPU readback | Passed; three cases byte-equal to CPU, rotated case mean byte error 0.0008544921875 |

A release-mode macOS build of the console FFI configuration also ran Koikake
for 3,600 fixed 16 ms frames, through its title, GAME START click, and ADV text.
It pulled 768 stereo audio frames per VM frame, observed nonzero PCM on 2,181
frames, and verified silence after destroying the engine while retaining the
sink. Screenshots were inspected. This checks the shared core and C ABI; it is
not a console-device or audible-output test. Resources came from the isolated
root under `target/reverse-audit`, with its own save directory. Logs and the
latest log and summary are `fx-ffi-smoke.log` and `console-ffi-report.json`
there. A second 1,800-frame run validated the native scene ABI with the same
game, including texture pointers/generations, draw references, and audio. See
[implementation evidence](../reverse/fx-vm-implementation.md) for scope and
remaining shader/VM compatibility work.

```sh
cargo test -p pal-vm --lib --no-default-features --features host-audio
cc -std=c11 -Wall -Wextra -Werror platform/common/test_console.c -lm -o /tmp/sena-console-test
/tmp/sena-console-test
```
