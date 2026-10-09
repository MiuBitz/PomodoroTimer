# Contributing

Build on Windows with Rust, Microsoft C++ Build Tools, and WebView2.
Keep the HTML/CSS/JavaScript frontend buildless and its fonts and icons local.

Before submitting a change:
- Run cargo fmt --check.
- Run cargo test --locked.
- Run build_windows.bat for changes affecting the app.
- Check the timer and Settings at narrow window sizes.
- For website changes, preview docs/index.html on desktop and mobile.

The optional Playwright checks in tests use the installed Microsoft Edge browser. Install Playwright locally without saving a package manifest, or point POMODORO_NODE_MODULES at a directory containing the package.

Keep personal settings, target directories, and test data out of commits. Update screenshots and the portable executable when app behavior or visuals change.
