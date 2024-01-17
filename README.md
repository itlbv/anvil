# anvil

Built with SDL2:
https://github.com/rust-sdl2/rust-sdl2

In order to build the project, first install SDL2 with related libraries:
```
brew install sdl2 
brew install sdl2_ttf
```

In case of linkage errors, get the location of brew installed libraries `echo $(brew --prefix)/lib` and add it to LIBRARY_PATH. It could look something like this:

`export LIBRARY_PATH="$LIBRARY_PATH:/opt/homebrew/lib"`

you may also add this line directly to your shell config (`~/.zshenv` or `~/.bash_profile`)

To make it work in Intellij IDE go to Edit Configurations and add `:/opt/homebrew/lib` or whatever `echo $(brew --prefix)` returns you to LIBRARY_PATH in your running configuration. 