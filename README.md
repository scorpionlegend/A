# A Language

A is a small, evolving programming language with a simple CLI.

## Install (from a release package)

Download the release package for your OS and extract it. Then run the installer:

Windows (user install):

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1 -Scope User
```

Windows (all users, admin required):

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1 -Scope AllUsers
```

macOS / Linux (user install):

```bash
bash install.sh --scope user
```

macOS / Linux (all users, admin required):

```bash
sudo bash install.sh --scope system
```

This installs the `a` executable into your PATH.

## Usage

Show CLI help:

```bash
a --help
```

Show help for a subcommand:

```bash
a run --help
a build --help
a update --help
```

Run a source file:

```bash
a run file.a
```

Run a source file and force recompilation (ignore existing bytecode):

```bash
a run file.a --fresh
```

Build bytecode:

```bash
a build file.a
```

Build with custom output:

```bash
a build file.a --out path/to/custom.a.byte
```

Build and run in one step:

```bash
a build file.a --run
```

This creates `file.a.byte`.

Run bytecode:

```bash
a run file.a.byte
```

Update from GitHub Releases (public repo):

```bash
a update --repo owner/name
```

Check for updates without downloading:

```bash
a update --repo owner/name --check
```

Or set the repo once:

```bash
# Windows (PowerShell)
$env:A_UPDATE_REPO = "owner/name"

# macOS / Linux
export A_UPDATE_REPO="owner/name"
```

Update expects release assets named:
- `a-windows-x86_64.exe`
- `a-macos-x86_64`
- `a-linux-x86_64`

## Example

```a
Func main() {
    Write("Hello")
}
```

## Notes

- Source files use the `.a` extension.
- Bytecode files use the `.a.byte` extension.
