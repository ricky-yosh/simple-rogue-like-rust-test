# Setup and first run

Bevy is installed into a project through Cargo, Rust's package manager and build
tool. There is no separate engine application required for this plan.

## What is already available

Read-only checks on the machine used for this research found:

| Item | Observation |
| --- | --- |
| Platform | macOS / Darwin, Apple Silicon (`arm64`) |
| Rust toolchain manager | `rustup 1.29.2` |
| Compiler | `rustc 1.99.1` |
| Build tool | `cargo 1.99.1` |
| Apple development tools | Xcode selected; `clang` found through `xcrun` |
| Rust components | rustfmt, Clippy, and rust-analyzer installed |

The compiler exceeds Bevy 0.20.1's declared minimum of Rust 1.97.1.
The application subsequently built successfully and selected the Metal renderer
on launch; visual confirmation remains pending. See the documentation index for checks run.
Another participant's machine may need different setup.

## Prerequisites on a new machine

1. Install stable Rust using the instructions at [rustup.rs](https://rustup.rs/).
2. Install the platform's native development tools:
   - **macOS:** Xcode or its Command Line Tools (`xcode-select --install` if absent).
   - **Windows:** Visual Studio C++ Build Tools with the Desktop development with
     C++ workload and Windows SDK.
   - **Linux:** install the distribution-specific compiler, windowing, audio, and
     device development packages from the official dependency guide linked below.
3. Use an editor with rust-analyzer, such as VS Code. It is helpful, not required.
4. Check `rustc --version`, `cargo --version`, and native tool availability.

Installers and system package changes are separate from editing this repository;
review them before running. No system tools were installed for this checkpoint.

## Project initialization reference

The repository is now initialized; use `cargo run` to launch it.
The following is the setup sequence for reference, not something to repeat here:

```sh
cargo init --bin --name simple_rogue_like_rust_test .
cargo run
cargo add bevy@0.20.1
```

The first `cargo run` should print Rust's generated greeting. Adding the dependency
alone does not create a game window: next replace the starter program with a
minimal Bevy app that installs the default engine plugins, then run `cargo run`
again. Add a 2D camera and a visible colored player shape as the first visual check.
Use the version-matched examples linked in the overview for the actual Rust code.

Initially use Bevy's defaults to reduce setup decisions. Once the baseline works,
consider disabling unused 3D/audio features if compile cost warrants it.

The official guide recommends these development profiles in `Cargo.toml`:

```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

This improves runtime performance while increasing dependency compilation cost.
Expect the first build to take substantially longer than a small Rust program.
Measure before adding dynamic linking, custom linkers, or nightly toolchains;
none is required to get started.

## First-run acceptance checklist

- [ ] `cargo run` builds and opens a window without panic.
- [ ] A colored player shape is visible through the 2D camera.
- [ ] Closing the window exits the application.
- [ ] Record toolchain, Bevy version, build duration, and any setup problems.
- [ ] Commit `Cargo.lock` and ignore generated `target/` output.

If compilation succeeds but launch fails, capture the error and distinguish
native-tool/linker problems from display/GPU problems. A headless test is not
evidence that the graphical application works.

## References

- [Official setup instructions](https://bevy.org/learn/quick-start/getting-started/setup/)
- [Linux dependency instructions](https://github.com/bevyengine/bevy/blob/v0.20.1/docs/linux_dependencies.md)
- [Rust learning resources](https://www.rust-lang.org/learn)
