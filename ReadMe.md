# eudaemon

**E**macs **U**ser **D**aemons

A kind spirit to help with managing Emacs client and server processes.

(A _very_ early work in progress 👼👹⚠)




## Usage

```
eud [list|new NAME|connect [NAME]|kill [NAME|--all]]
```

where:
`list` shows a list of all running Emacs daemons, with respective PIDs

`new NAME` creates a new Emacs daemon using a socket called NAME

`connect NAME FILE` creates a new Emacs client process (i.e. `emacsclient`) connected to the socket called NAME and visits FILE with Emacs; if no FILE is passed, Emacs will visit the working directory in `dired` mode. Exits and displays an error if FILE does not exist if a daemon process with socket NAME does not exist.

`kill NAME` sends a TERM signal (15) to the Emacs daemon process with socket NAME,

(TODO: or with `--all` sends the kill signal to all Emacs daemon processes)



## Notes

### Sockets live in `~/emacs.d/sockets` by default
This is only intended to work with Emacs server daemons with Unix socket files (i.e. not TCP). The socket files belonging to any Emacs daemons from `eud` are stored in a single directory (this shall be made configurable). While this is inconsistent with Emacs' own implementation (which allows for the user to set `server-socket-dir` in their Emacs configuration, _or_ for the socket directory to use the environment `$TMPDIR`, _or_ to fall back to the system default temp. directory (typically `/tmp/emacs$(id -u)`). Currently, the socket directory is set to `~/.emacs.d/sockets/` (which will be created by `eud` if it does not already exist). Using a single location for this, explicitly, has the pleasant side-effect of avoiding unix socket files being strewn around various temporary directories (as can happen when using [`nix-shell`](https://nixos.wiki/wiki/Development_environment_with_nix-shell) environments, for example).



## TODOs

[ ] hook for Emacs to 'know' where to look for `eud`'s sockets

[ ] `tokio::process` for spawning child processes and reading asynchronously from them

[ ] tests de unidad!

[ ] reasonable exit codes on error?

[ ] given the name, we should really ensure this works with [Doom Emacs](https://github.com/doomemacs/doomemacs)

[ ] nix.

[ ] MacOS launchd integration (set default server to launch on login/boot?)




