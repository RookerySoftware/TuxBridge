---
title: Python
summary: Run Python, use virtual environments, install packages with pip or uv, and Jupyter notebooks.
section: Programming
order: 32
---

# Python

Most Linux systems come with Python 3, because parts of the system use it. Type `python3`
(on Arch-based systems also `python`).

```bash
python3 --version
python3                 # an interactive prompt; exit() or Ctrl+D leaves
python3 script.py       # run a file
```

If it is missing: `sudo apt install python3` or `sudo pacman -S python`.

## Do not install packages into the system's Python

The system's Python belongs to the system. Newer Ubuntu, Debian and Arch refuse
`pip install something` with **externally-managed-environment** to protect it. Use one of these
instead:

| For | Use |
| --- | --- |
| A project's packages | A **virtual environment** (venv), or **uv** |
| A command-line tool written in Python (black, httpie…) | **pipx** or `uv tool install` |
| A package the system already offers | `sudo apt install python3-requests` / `sudo pacman -S python-requests` |

## Virtual environments

A virtual environment is a folder with its own Python packages, one per project.

```bash
sudo apt install python3-venv python3-pip     # Ubuntu, Mint, Debian (Arch has them built in)
cd ~/projects/my-app
python3 -m venv .venv                         # make it
source .venv/bin/activate                     # use it: the prompt shows (.venv)
pip install requests flask                    # packages go into .venv
pip freeze > requirements.txt                 # remember what you installed
deactivate                                    # leave it
```

Later, or on another PC: `python3 -m venv .venv && source .venv/bin/activate && pip install -r requirements.txt`.

## uv: fast and simple (recommended)

**uv** manages Python versions, virtual environments and packages, very quickly.

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh     # installs to ~/.local/bin
uv init my-app && cd my-app                         # a new project
uv add requests                                     # add a package (makes .venv for you)
uv run main.py                                      # run inside the project's environment
uv python install 3.13                              # a specific Python version
uv tool install ruff                                # a tool, available everywhere
```

## pipx: Python tools as commands

```bash
sudo apt install pipx          # or: sudo pacman -S python-pipx
pipx ensurepath                # adds ~/.local/bin to PATH (new terminal afterwards)
pipx install httpie
```

## Jupyter notebooks

```bash
uv tool install jupyterlab     # or inside a venv: pip install jupyterlab
jupyter lab
```

It prints a `http://localhost:8888/...` link: open it in your Windows browser. In VS Code
(connected to WSL), notebooks open directly with the Jupyter extension.

## Python with a window

Tkinter, Matplotlib windows, PyQt and pygame open as windows on your Windows desktop through
WSL. Tkinter may need `sudo apt install python3-tk` (Arch: `tk`).

## Machine learning and the GPU

PyTorch and TensorFlow can use an NVIDIA GPU inside WSL. Install the normal NVIDIA driver on
**Windows** only, then the CUDA build of the library in Linux. See
[Graphics, the GPU and Linux apps](gpu.md).

## Common problems

| Message | Fix |
| --- | --- |
| `externally-managed-environment` | Use a venv, uv or pipx (above). Never use `--break-system-packages` on the system Python. |
| `No module named venv` / `ensurepip is not available` | `sudo apt install python3-venv` |
| `command not found: python` | Use `python3`, or `sudo apt install python-is-python3` |
| `pip: command not found` | Inside a venv it exists; system-wide, `sudo apt install python3-pip` |
| Packages install but aren't found | The venv isn't active (`source .venv/bin/activate`), or VS Code uses another interpreter (Ctrl+Shift+P → *Python: Select Interpreter*). |
