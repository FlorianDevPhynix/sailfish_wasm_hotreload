# WIP sailfish wasm hotreload

This is a test repository to figure out how hotreloading compiled templates in rust is going to work.

# Idea

A macro generates render methods, that get a wasm vm instance from a global static store.
Using this vm a function will be execute. The Template itself will be passed as an input parameter (serialized to bytes).
When compiling to webassembly, this macro will instead generate a different function.
This is the one that gets executed by the generated render methods.
A watcher is used to re-compile changes and the vm will be replaced in the global static store.
