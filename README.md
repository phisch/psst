# Hush

A not-so-ugly replacement for your pinentry and keyring prompter for wayland using a layer-shell overlay.

- **`hush-pinentry`**: the dialog GnuPG uses to ask for your key passphrase or smartcard PIN (a *pinentry* program for `gpg-agent`).
- **`hush-keyring-prompter`**: the dialog that unlocks your GNOME keyring, replacing the standard gnome-keyring/gcr prompt.

## Setup

Build the two programs:

```sh
cargo build --release
```

Both binaries land in `target/release/`.

**Use it for GnuPG**: point `gpg-agent` at it in `~/.gnupg/gpg-agent.conf`:

```
pinentry-program /path/to/hush/target/release/hush-pinentry
```

then reload the agent:

```sh
gpg-connect-agent reloadagent /bye
```

**Use it for the GNOME keyring**: run the prompter (e.g. from yourcompositor's autostart):

```sh
hush-keyring-prompter
```

It takes over keyring unlock prompts for as long as it's running.
