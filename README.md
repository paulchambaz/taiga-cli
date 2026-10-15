# taiga-cli

taiga-cli is a cli tool for taiga.io inspired by taskwarrior. It is designed to facilitate the use of kanbans from your terminal without having to open the web interface.

![](./demo.gif)

That being said, the real ground truth is taiga.io, and large operations (such as creating new projects, adding new members or creating new kanban statuses are not supported by this simple cli tool. taiga-cli is a lightweight interface, mostly around the user stories of taiga (the tasks we add to the kanban). taiga-cli offers powerful command to filter the tasks from your terminal and to modify them.

## Installation

### Nix

```sh
nix profile install github:paulchambaz/taiga-cli
```

Or add `github:paulchambaz/taiga-cli` as an input of your flake and install its `packages.default`. The man page and the shell completions are installed with it.

### Cargo

```sh
cargo install taiga-cli
```

### Manual

See the [**Building**](#building) section.

## Usage

Login once, then every project is a command:

```sh
taiga login
taiga demo                                  # list the tasks
taiga demo new fix the login @me due:eow +team
taiga demo 3 move in-progress
taiga demo mod 3 -due +block better title
taiga demo @me due:eow                      # my tasks due this week
taiga demo 3 subtask new write the tests
taiga demo burndown
```

Task numbers are the positions in the last list printed. Everything is detailed in the [`man`](./taiga.1.scd) page (`man taiga` once installed): commands, modifiers, filters, dates, subtasks and burndown.

Completions for zsh, bash and fish are in [`completions/`](./completions) and follow your projects, statuses, sprints and members. See the man page for how to enable them.

Your session is kept in `~/.cache/taiga/config` (readable only by you), password included, so it can be renewed. Delete the file to log out.

## Building

### Nix

```sh
git clone https://github.com/paulchambaz/taiga-cli.git
cd taiga-cli
nix develop # a shell with every dev tool
nix build
just --list # the dev commands
```

### Manual

You need `cargo`, `scdoc` for the man page and `just` for the dev commands.

```sh
just build
just test
just lint
just install # binary, man page and completions in ~/.local
```

## Contribution

Contributions are welcome: feature suggestions, bug reports or code, through issues and pull requests on GitHub.

## License

<a href="https://www.gnu.org/licenses/gpl-3.0.en.html">
  <img src="https://www.gnu.org/graphics/gplv3-127x51.png" alt="GNU GPLv3" />
</a>

taiga-cli is free software. You can use, study, modify and distribute it under the
terms of the [GNU General Public License](https://www.gnu.org/licenses/gpl-3.0.en.html),
version 3.
