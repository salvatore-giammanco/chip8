## Building on OSX

```
brew install sdl2
sudo chown root:wheel /usr/local/bin/brew
sudo brew link sdl2
```

Add this to bash profile:
```
export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
```