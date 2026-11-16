# wyc-lib — pre-Foundation Node package

This tarball contains one native binary for the OS/architecture in package.json.
Use Node 22.22.0 on Windows x64 MSVC or Ubuntu 24.04 x64 GNU, installing the
matching build. There is no download or compilation during installation.
The package is private and has not been published to npm.

Install the matching tarball with `npm install --engine-strict ./wyc-lib-0.1.0.tgz`.
Import with `require('wyc-lib')`. Public typed APIs: analyze_batch,
calculate_indicators, analyze_structure and AnalysisState (result, identity,
update_closed_bar, reset, close). JSON pilot APIs remain available.
Inputs and outputs are owned copies; calls are synchronous.

For Next.js, use the Node.js server runtime and
`serverExternalPackages: ['wyc-lib']`. Build and start with the native package
installed on the target OS. Browser, client components, Edge, WASM and other
OS/CPU/Node versions have not been validated. This artifact is not a portable
multi-platform installer or a Foundation release.
