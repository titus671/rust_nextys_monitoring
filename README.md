# Cross Compiling

I'm using cross to cross compile to `armv7`.
The command is `cross build --target armv7-unknown-linux-gnueabihf`

# State

The binary is now in a state where it could be run in production.
Ideally I would like to rewrite some of the library to expose more of a bus with
the intention you could control it from an external client such as a webserver.
