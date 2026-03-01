# gurkan's ttcore preparer tool

> [!IMPORTANT]
> you need ffmpeg installed for this tool to work (the credit burn-in)

> [!NOTE]
> this is mainly intended for private usage, so don't expect this tool to be very useful for other tasks

this automates the tedious parts of creating my traitor town core videos by doing the following

- downloads the clips marked as selected from https://ttcore.gurkz.me
- burns in the credits onto the video


## Usage

make a config.toml (or pass a file using the --config option) file with the following content

```toml
[api]
key = "" # https://ttcore.gurkz.me/user to get an api key (only I can do this)

[fs]
out_dir = "out" # just an example
font_file = "./data/space-grotesk.ttf" # just an example
```

run `ttcore-clip-preparer download --video-id <video id>` to download the clips

and run `ttcore-clip-preparer burn-credits --video-id <video id>` to burn the credits in
