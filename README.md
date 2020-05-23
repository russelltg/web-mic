# web-mic

Ever wished you could use your phone mic from your linux computer (exposed as a PulseAudio source)?

This provides a simple solution to this problem, all wrapped in a single staticly-linked binary written in rust without unsafe code.

# Installation

```bash
cargo install --git https://github.com/russelltg/web-mic
```

# Usage

Run `web-mic`, and connect to it on your phone with `https://<your ip>:8000`. Note the `s`--it must be https. 
You will see a security warning, choose "advanced" then "proceed to xxx". If you are curious about this warning,
see [below](#TLS-Requirement)


# Limitations

## TLS requirement

The Web Audio APIs [_only work in secure contexts_](https://developer.mozilla.org/en-US/docs/Web/API/AudioWorkletNode), so it runs using self-signed certificates. 
However, these certificates are cached (in `~/.cache/web_mic`), so you should only have to bypass the security warning once.

## Focus

At least on android chrome, it seems to throttle the capture worker when the phone is locked or similar, so you need to disable the screen timeout.
I personally have the setting in developer settings turned on to not timeout the screen when its plugged in.

## Latency

This app should obviously not be used when low latency is required. In my limited test, I got around 100ms of latency, which is fine for VoIP.

## Upgrade from HTTP to HTTPS

Unfortuantely, currently you can't just put your local IP in the phone browser, as it defaults to HTTP, and [warp currently cannot upgrade from HTTP to HTTPS on the same port](https://github.com/seanmonstar/warp/pull/431).

This could be worked around by opening a new port that just accepts redirects to https, but for me a bookmark fixes that problem fine. If someone wants to implement that PRs are welcome.

# Contributing

This app is mostly for personal use, so reach out to me before spending a lot of time on a feature. Otherwise, pull requests are welcome!
