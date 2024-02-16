# taiga-cli

taiga-cli is a cli tool for taiga.io inspired by taskwarrior. It is designed to facilitate the use of kanbans from your terminal without having to open the web interface.

![](./demo.gif)

That being said, the real ground truth is taiga.io, and large operations (such as creating new projects, adding new members or creating new kanban statuses are not supported by this simple cli tool. taiga-cli is a lightweight interface, mostly around the user stories of taiga (the tasks we add to the kanban). taiga-cli offers powerful command to filter the tasks from your terminal and to modify them.

## Installation

There are a couple ways to install this project. First, it is hosted on crates.io, i have also added it to nixpkgs and on the arch user repository. Of course, you can always install it manually.

### Manual

To install the project manually, please consult the [**Building**](#Building) section.

### Nix

You can try this program in a `nix-shell`:

```sh
nix-shell -p taiga-cli
```

You can then add it to your `configuration.nix` if you're happy with it.

### Cargo

```sh
cargo install taiga-cli
```

### AUR

```
yay -S taiga-cli
```

## Usage

To understand how to use this program, it is recommended to read the [`man`](./taiga.1.scd) page. All instructions are detailed there.

Here is a brief overview of the program :

```sh
taiga --help
Cli tool for taiga.io

Usage: taiga [PROJECT] [COMMAND] <OPTIONS>
                     
Commands:            
  login              Login to a taiga instance
  projects           Refresh and print the project list
                     
Projects:            
  demo  Run command on demo
                     
Options:             
  --help             Print the help message and exit
  --version          Print the version and exit
```

*Do note that demo is a placehold and that your actual taiga project will be listed in the projects section.*

## Building

### Nix

This project uses [nix](https://github.com/NixOS/nix) for development. If you want to contribute, it is recommended to install nix (not NixOS) to access the development shell.

```sh
git clone https://github.com/paulchambaz/taiga-cli.git
cd taiga-cli
nix develop
nix build # to build the project
nix shell # to enter a shell where the built project is installed
just --list # to list the dev commands
```

### Manual

If you want to manually build the project, it isn't to hard either.

You will need `scdoc` in order to build the man page and `just` to run the dev commands.

```sh
just run
just build
just fmt
just coverage
just watch-test
```

## Contribution

Contributions to taiga-cli are welcome. Whether it's feature suggestions, bug reports, or code contributions, feel free to share your input. Please use the project GitHub repository to open issues or submit pull requests.

## License

taiga-cli is released under the GPLv3 License. For more details, refer to the LICENSE file included in the repository.
