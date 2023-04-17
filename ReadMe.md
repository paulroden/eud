## Usage

```
eud [list|new NAME|connect [NAME]|kill [NAME|--all]]
```

where:
`list` shows a list of all running Emacs daemons, with respective PIDs

`new NAME` creates a new Emacs daemon using a socket called NAME

`connect [NAME]` creates a new Emacs client process connected to the socket called NAME; if NAME does not already exist, an Emacs daemon process is created called NAME

`kill NAME` sends a kill (?) signal to the Emacs daemon process with socket NAME, or with `--all` sends the kill signal to all Emacs daemon processes.
