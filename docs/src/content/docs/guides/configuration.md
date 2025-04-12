---
title: Configuration
description: Configure `jjj` to your liking
---

`jjj` piggybacks off Jujutsu's configuration, allowing you to easily make
changes at the user and repo levels. You can run the following to set values:

```sh
# Example: Skip the splash screen on startup
jj config set --user jjj.splash.skip true
```

## Logging

Configure how `jjj` fetches logs from Jujutsu.

- **`jjj.log.poll_interval_ms`**: How frequently `jjj` should check the status
  of the current repository. Defaults to `1000`.

## Splash screen

Configure `jjj`'s splash screen.

- **`jjj.splash.skip`**: Whether to skip the splash screen on startup. Defaults
  to `false`.
- **`jjj.splash.total_duration_ms`**: The total max duration the splash screen
  should be displayed for. Defaults to`1950`.
- **`jjj.splash.line_interval_ms`**: The duration between splash screen
  animation frames. Defaults to`150` .
