# Releasing mga

mga is used by the OpenShift CI infrastructure (see [gather-must-gather-commands.sh](https://github.com/openshift/release/blob/master/ci-operator/step-registry/gather/must-gather/gather-must-gather-commands.sh)),
as such it should be released in a manner that can be pulled by that tooling.
To create a release do the following:

1. update `Cargo.toml` version if needed
2. run `./build-release-artifacts`
3. review the generated artifacts in `releases/<version>/`
4. optionally run `./build-release-artifacts --publish` to create the GitHub release for `v<version>` from the existing artifacts

This creates:
- `mga-<version>-darwin-arm64.tar.gz`
- `mga-<version>-linux-x86_64.tar.gz`
- `SHA256SUMS`

`--publish` is publish-only. It does not rebuild artifacts or update checksums, so run `./build-release-artifacts` first and review `releases/<version>/`.

The Linux artifact is built as a static `x86_64-unknown-linux-musl` binary via Podman.

## Notes from elmiko

Doing the release has become complicated, mainly because it needs to be compiled using an image which doesn't
bring in a glibc version greater than what is available in the image that will be run by OpenShift CI.
This poses a challenge when building the output. What I have traditionally done is to create an image
based on the same one used by the release tooling (see `hack/Dockerfile` for inspiration), and then run
that image while mounting the local directory and issuing the `cargo build --release` command. This
process is fragile and could use some improvement, but has been working so far.

In other words, beware, dragons ahead...
