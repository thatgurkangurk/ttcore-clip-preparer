# gurkan's ttcore preparer tool

**note!::** this is mainly intended for private use

this automates the tedious parts of creating my traitor town core videos by doing the following

- downloads the clips marked as selected from https://ttcore.gurkz.me
- burns in the credits onto the video


## Usage

No pre-built binaries will be provided, compile them yourself.

make a config.toml file with the following content

```toml
[api]
key = "" # https://ttcore.gurkz.me/user to get an api key (only I can do this)

[fs]
out_dir = "out" # just an example
font_file = "./data/space-grotesk.ttf" # just an example
```

run `ttcore-clip-preparer download --video-id <video id>` to download the clips

and run `ttcore-clip-preparer burn-credits --video-id <video id>` to burn the credits in