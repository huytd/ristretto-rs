# Ristretto: An opinionated blog engine

This is the blog engine that powered my blog (https://thefullsnack.com)

**Please note that this is not a static blog engine.**

## How to use?

First, compile the server. You'll need to install Rust (https://rustup.rs/).

```
cd generator-rs
cargo build
```

The executable binary will be available at `./generator-rs/target/debug/generator-rs` and you can run it via the `gen` symlink in the root folder.

```
./gen
```

Make sure you have configured the `.env` file properly:

```
DATE_FORMAT="%d-%m-%Y"
DOMAIN_NAME="https://thefullsnack.com"
RSS_TITLE="The Full Snack Blog"
RSS_DESCRIPTION="The Full Snack Blog RSS Feed"
```

The server wil be available at https://localhost:3123 by default.

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
