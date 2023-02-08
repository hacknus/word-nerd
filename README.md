# Word Nerd
A cross-platform speed reader written in rust.  
![Screenshot of the application on macOS](screenshot.png)  

The source code can be run using ```cargo run``` or bundled to a platform-executable using ```cargo bundle```.  
Currently [cargo bundle](https://github.com/burtonageo/cargo-bundle) only supports linux and macOS bundles [see github issue](https://github.com/burtonageo/cargo-bundle/issues/77).
As a work-around we can use [cargo wix](https://github.com/volks73/cargo-wix) to create a windows installer.  
It can be compiled and run on all platforms.
Tested on:
- MacOS 12.4 Monterey x86
- ...  

On Debian 12 (Testing) the following error occurred:
```
Error: glib-2.0 was not found in the pkg-config search path.
```
solved through
```
sudo apt-get install libgtk-3-dev
```

One might have to delete the ```Cargo.lock``` file before compiling.  
