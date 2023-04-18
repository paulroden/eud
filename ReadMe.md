# eudaemon

**E**macs **U**ser **D**aemons

A kind spirit to help with managing Emacs client and server processes.

(A _very, very_ early work in progress 👼👹⚠)


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



### TODOs

[ ] config via file and env (via monoidal overlay)

[ ] `emacsclient` processes to be spawned in background.

[ ] correct exit codes on error

[ ] tokio for spawning child processes and reading asynchronously from them

[ ] check and, when necessary, clean up redundant socket files in `TMPDIR` (safely)

[ ] given the name, we should really ensure this works with [Doom Emacs](https://github.com/doomemacs/doomemacs)

