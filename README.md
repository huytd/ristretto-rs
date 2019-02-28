# Ristretto: An opinionated blog engine

This is the blog engine that powered my blog (https://thefullsnack.com)

## How to use?

First, compile the generator. You'll need to install Rust (https://rustup.rs/).

```
cd generator-rs
cargo build
```

The executable binary will be available at `./generator-rs/target/debug/generator-rs` and you can run it via the `gen` symlink in the root folder.

```
./gen posts
```

## How to write post?

All posts should be located in `./posts` folder, in markdown format. Each post should starts with some metadata:

**posts/new-post.md**
```
---
title: <string>
published: true | false | private | guest
date: YYYY-DD-mm HH:MM:SS
tags: <string>, <string>,...
description: <string>
image: <url to a featured image>
---
```

## How do I publish a post?

Make sure your `published` field is set to `true` or `private` (generated but not show in homepage) or `guest` (shown as guest post).

Run the following command:

```
./gen posts
```

## Can I preview my post while writing?

Run the previewer:

```
./gen preview
```

Then you can go to `http://localhost:3123/view/<file-name-without-the-extension>`, for example: `http://localhost:3123/view/life-with-robot`.
