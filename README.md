<h1 align="center">git-ignore</h1>
<p align="center">
    <a href="https://github.com/sondr3/git-ignore/actions"><img alt="GitHub Actions Status" src="https://github.com/sondr3/git-ignore/workflows/pipeline/badge.svg" /></a>
    <a href="https://crates.io/crates/git-ignore-generator"><img alt="Crates" src="https://img.shields.io/crates/v/git-ignore-generator.svg" /></a>
</p>

<p align="center">
    <b>Create .gitignores with templates from www.gitignore.io, your own aliases and templates</b>
</p>

- **Simple**: `git ignore node` to print the `node` template.
- **Offline first**: Automatically caches templates for offline support.
- **Aliases, templates**: Create aliases for commonly combined templates, or make your own custom ones.
- **Magic**: Automatically generate your `.gitignore` by matching common files against templates.

<details>
<summary>Table of Contents</summary>
<br />

- [What and why](#what-and-why)
- [Installation](#installation)
- [Usage](#usage)
    - [Aliases](#aliases)
    - [Templates](#templates)
    - [Configuration](#configuration)
- [License](#license)

</details>

# What and why

Tired of visiting [gitignore.io](https://www.gitignore.io/) to get templates for your
`.gitignore` all the time? I was. So I [automated](https://xkcd.com/1319/) [it](https://xkcd.com/1205/).

`git ignore` allows you to easily and quickly get all the available templates from
[gitignore.io](https://www.gitignore.io/), even while offline. You can also define
your own aliases for common combinations of templates, or create your own completely
custom ones for even more power.

<details>
<summary>Demo</summary>

[![asciicast](https://asciinema.org/a/454912.svg)](https://asciinema.org/a/454912)

</details>

# Installation

Currently, the package is available a couple of places, including Homebrew, AUR and Nix.

<dl>
  <dt>Cargo</dt>
  <dd><code>cargo install git-ignore-generator</code></dd>

  <dt>Nix</dt>
  <dd><code>nix-env -iA nixpkgs.gitAndTools.git-ignore</code></dd>

  <dt>Homebrew</dt>
  <dd><code>brew install git-ignore</code></dd>

  <dt>Arch (replace <code>paru</code> with you favorite AUR tool)</dt>
  <dd><code>paru git-ignore-bin</code></dd>
  <dd><code>paru git-ignore</code></dd>
</dl>

## Release pages

You can also download the matching release from the [release
tab](https://github.com/sondr3/git-ignore/releases), extracting the archive and
placing the binary in your `$PATH`. Note that for Linux the
`unknown-linux-musl.tar.gz` is preferred as it is statically linked and thus
should run on any Linux distribution.

# Usage

**NOTE:** Similar to the `nix-search` command, this program prints a message
to `stderr` about using cached results. This does _not_ interfere with piping
and is purely informational. You can also optionally use `--write` to automatically
write the resulting ignores to `$CWD/.gitignore` instead of piping.

## Updating templates

To download and cache all available templates, use `--update`. This can also be
used in combination with any of the other flags/arguments, or be run as a
standalone flag.

```sh
$ git ignore -u
Info: Update successful
```

## Automatic matching

By matching against project or language specific files and extensions you can have
your `.gitignore` automatically generated for you. Do you have a `package.json` and
`Cargo.toml` in the current directory? `--auto` will automatically add `node` and `rust`
to the template output.

```sh
$ git ignore -a

### Created by https://www.gitignore.io
### Rust ###

[...]

# These are backup files generated by rustfmt
**/*.rs.bk
```

## List templates

To list all the available templates:

```sh
$ git ignore --list
  1c
  1c-bitrix
  a-frame
  actionscript
  ada
  [...]
  zukencr8000
```

The `--list` option is also used to search for templates matching your input. The
matching is done by doing `template.contains(phrase)`, so searching for `intellij`
will list all templates containing that phrase. You can also search for multiple
templates at once:

```sh
$ git ignore -l rust intellij

  intellij
  intellij+all
  intellij+iml
  rust
```

## Printing templates

Once you've found your templates, you can print them by omitting `-l|--list`. **Note:**
listing and searching for templates is inexact, but printing them requires exact matches.

```sh
$ git ignore rust intellij+all

### Created by https://www.gitignore.io
### Rust ###

[...]

# These are backup files generated by rustfmt
**/*.rs.bk
```

## Aliases

Aliases are a way to combine common combinations of templates, if you find
yourself always using `node` and `visualstudiocode` in your frontend projects
you can create an alias for it for ease of access. Aliases have higher priority
than templates from www.gitignore.io, so an alias named `node` will be used
instead of the template. When listing all available templates, aliases are colored
yellow to allow you to distinguish them from regular templates.

### Listing

```sh
$ git ignore alias list
Available aliases:
node => ["node", "nextjs", "visualstudiocode"]
rust => ["rust", "intellij+all"]
```

### Adding

```sh
$ git ignore alias add node node nextjs visualstudiocode
Created alias node for ["node", "nextjs", "visualstudiocode"]
```

### Removing

```sh
$ git ignore alias remove node
Removed alias node
```

## Templates

Templates are custom templates created by you for things that do not have an
existing template defined. When listing and searching templates has the highest
priority (`templates > alias > normal`). Templates are listed with a blue color
to distinguish them from aliases and normal templates.

### Listing

The file name is the name of the file in `$HOME/.config/git-ignore/templates`.

```sh
$ git ignore template list
Available templates:
docs => "docs.txt"
```

### Adding

All templates are created in `$HOME/.config/git-ignore/templates`. So the name
you give for the file is the filename that is used in this directory.

```sh
$ git ignore template add docs docs.txt
Created template docs at ~/.config/git-config/templates/docs.txt
```

### Removing

```sh
$ git ignore template remove node
Removed template node
```

## Configuration

You can create the configuration file and directories by running `git ignore init`. This
will create `$HOME/.config/git-ignore/config.toml` and `$HOME/.config/git-ignore/templates/`.

The config file is a simple [TOML](https://toml.io/en/) file:

```toml
[aliases]
node = [
    'node',
    'nextjs',
    'visualstudiocode',
]

[templates]
docs = 'docs.txt'
```

## Completion

If your method of installation didn't include shell completion, you can manually
source or save them with the `git ignore completion <shell>` command.

## Help

Finally, help is always available with `git ignore help`/`git ignore -h` (or `--help` if your installation
included man pages).

# LICENSE

GPLv3+.
