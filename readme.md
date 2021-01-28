# Molecular Disruption Device

A simple HTML / CSS terminal-based stress testing tool, written in Rust!

## Execute
You're required to either have NodeJS & Yarn installed on the system to handle the native nodejs utilites included with Tauri (and the buildscript).
It uses the native windows webview, so currently windows is the only platform this is able to run on (rip linux)

To build the program, ``yarn tauri build``

To run the development build ``yarn tauri dev``

## Frontend
The frontend has gone through several interations and revisions to make it usable, landing somewhere inbetween slick and functional (but still doesn't look nice).
The main rendered file is ``dist/index.html``, (index.tauri.html is just a build artifact). Inside it contains all the Html, Css, and JS.

The rendered webview frontend communicates with the backend via a websocket connection, constructed by the web server. The web view also handles basic commands like ``clr``, without passing them to the backend.


## Backend
The backend is mostloy lcated in src/coms & src/traffic,

udp & tcp are very simple wrappers around the standard udp & tcp socket libraries, providing custom structures to reduce the complexity a little bit.

command_handler contains each command the program is capable of running, (able to be listed in the terminal by typing "help"). Each command can have it's help or usage displayed by typing the command with no additional arguments.

config_handler is just a simple struct for handling the one (and only) configurable option, the payload message that's sent with tcp, udp, & http requests.

## Testing

server/index is a simple nodejs http & tcp server to test the performance of this stress tester via logging all successful socket messages or requests