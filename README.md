# LemonCraft

LemonCraft is a voxel action-adventure RPG set in a vast fantasy world. It is
derived from [Veloren](https://gitlab.com/veloren/veloren) (GPL-3.0), rewritten
as an independent project with a streamlined, focused gameplay vision.

## Development

LemonCraft is developed in the open on [GitHub](https://github.com/LemonHub-org/craft).

- Code: this repository (`main` branch)
- Issues and discussion: GitHub issues

We welcome contributions — code, art, music, and feedback.

## Building and running

The project is a Rust workspace (nightly toolchain, pinned in
`rust-toolchain`). See the upstream Veloren [book](https://book.veloren.net/contributors/introduction.html)
for build prerequisites.

```sh
# Client (needs assets/)
cargo run --bin lemoncraft-voxygen
# Server
cargo run --bin lemoncraft-server-cli
```

Handy aliases from `.cargo/config.toml`: `cargo test-voxygen`,
`cargo test-server`, `cargo server`, `cargo swarm`, `cargo img-export`.

## Content packs

LemonCraft keeps gameplay lean by default. Full content is available as
optional [content packs](docs/content-packs.md) — the same mechanism future
mods will use. Data-driven RON assets remain fully moddable.

## FAQ

### **Q:** How is this game licensed?

**A:** **It is free to play, modify and distribute. Forever.**
LemonCraft is licensed under the **[GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0-standalone.html)**,
as is the Veloren project it derives from.

### **Q:** What platforms are supported?

**A:** LemonCraft can run on Windows, macOS and Linux on a great range of CPU
architectures. x86_64 is the main focus in development.

### **Q:** How does LemonCraft relate to Veloren?

**A:** LemonCraft started as a fork of Veloren 0.18 and has diverged: it has its
own name, brand, and a streamlined gameplay direction. Upstream policies do not
apply here. Credit for the original game goes to the Veloren community.

## Credit

LemonCraft is built on the work of the Veloren community — its developers,
artists, musicians, and translators. Thank you.
