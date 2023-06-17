# factorio-server

This is [Factorio](https://factorio.com) packaged in a container image.
There are two images. One is fairly minimal and doesn't contain much other than the headless server.
The other contains an application which can be used to install mods and start the server which is built using the rust project in this repository.

For my uses, I wrote a nomad jobspec to run this container.
You may find the jobspec in [my nixos configuration](https://github.com/justinrubek/nixos-configs/blob/71c4f632eebc400740826aa2cafcb271ebc29b9e/nomad/jobs/factorio.nix).

If you're looking for a more full-featured image, you may want to check out [factorio-docker](https://github.com/factoriotools/factorio-docker).
This one is customized to my use cases, but could be very useful to you.
