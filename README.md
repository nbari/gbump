# gbump
Git tag semantic version bumper

[![crates.io](https://img.shields.io/crates/v/gbump.svg)](https://crates.io/crates/gbump)
[![Build Status](https://travis-ci.org/nbari/gbump.svg?branch=master)](https://travis-ci.org/nbari/gbump)

What does it do?
================

Will print current version if any and bumped version

How to use it?
==============

To install:

    cargo install gbump

You can copy `gbump` to `/usr/local/bin/gbump` or somewhere available in your path

Available options are: `patch`, `minor`, `major`. (defaults to patch)

For example if current version tag is `0.1.1`:

Using `patch` will bump `0.1.1` to `0.1.2`

    $ gbump patch
    0.1.1 --> 0.1.2

Using `minor` will bump `0.1.1` to `0.2.0`

    $ gbump minor
    0.1.1 --> 0.2.0

Using `major` will bump `0.1.1` to `1.0.0`

    $ gbump major
    0.1.1 --> 1.0.0
