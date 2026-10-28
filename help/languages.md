---
title: Other programming languages
summary: Set up C and C++, Rust, Go, Java, .NET, Ruby, PHP and more, on apt and pacman systems.
section: Programming
order: 34
---

# Other programming languages

Linux is a natural home for programming: compilers and tools install with one command. Where a
language has its own installer, it usually gets newer versions than the package manager.

## C and C++

```bash
sudo apt install build-essential gdb cmake ninja-build     # Ubuntu, Mint, Debian
sudo pacman -S base-devel gdb cmake ninja                  # Arch, CachyOS, Omarchy
```

```bash
gcc hello.c -o hello && ./hello
g++ -std=c++20 -O2 main.cpp -o main
```

`clang` and `clangd` (for editors) install with `sudo apt install clang clangd` or
`sudo pacman -S clang`. `make` and `pkg-config` come with the packages above.

## Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# open a new terminal
cargo new hello && cd hello && cargo run
rustup update            # newer Rust later
```

Rust needs a C linker: install `build-essential` (apt) or `base-devel` (pacman) first.

## Go

```bash
sudo apt install golang-go     # may be older; or download from go.dev/dl
sudo pacman -S go
go version
```

From go.dev: unpack into `/usr/local/go` and add `export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin`
to `~/.bashrc`.

## Java

```bash
sudo apt install openjdk-21-jdk maven       # the version number may differ: apt search openjdk
sudo pacman -S jdk-openjdk maven
java -version
```

**SDKMAN!** (`curl -s "https://get.sdkman.io" | bash`) installs and switches Java versions,
Gradle, Kotlin and more.

## .NET (C#, F#)

```bash
sudo apt install dotnet-sdk-8.0     # Ubuntu; check apt search dotnet-sdk for newer
sudo pacman -S dotnet-sdk
dotnet new console -o hello && cd hello && dotnet run
```

Microsoft's `dotnet-install.sh` script installs any version into your home folder.

## Ruby

```bash
sudo apt install ruby-full      # or: sudo pacman -S ruby
```

For projects (Rails), a version manager like **rbenv** or **mise** is easier.

## PHP

```bash
sudo apt install php-cli php-mbstring php-xml composer
sudo pacman -S php composer
php -S localhost:8000            # a quick web server for the current folder
```

## Many languages, one tool: mise

**mise** installs and switches versions of Node, Python, Go, Ruby, Java and many more, per
project:

```bash
curl https://mise.run | sh
echo 'eval "$(~/.local/bin/mise activate bash)"' >> ~/.bashrc
# new terminal
mise use --global node@lts python@3.13
```

## Databases

The easiest way to run PostgreSQL, MySQL, Redis or MongoDB for development is **Docker**: one
command, nothing installed into the system, and removed as easily (see [Docker](docker.md)).
They can also be installed directly, for example `sudo apt install postgresql` and
`sudo systemctl enable --now postgresql`.

## Tips for every language

- Keep projects in Linux (`~/projects`), not under `/mnt/c`: builds are much faster.
- Use [VS Code with Linux](vscode.md) and install the language's extension *in WSL*.
- A program that serves on a port is reachable from Windows at `http://localhost:<port>`.
