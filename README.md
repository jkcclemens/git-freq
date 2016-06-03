# gitfreq

Adds a `git freq` command to use with [spark](https://github.com/holman/spark).

For example:

```sh
git freq $(date '+%Y-%m-%d') -7 | spark
```

That creates a sparkline for the last seven days of activity in a git repo.

## Installation

**crates.io version**

```sh
cargo install git-freq
```

**Cloned version**

```sh
git clone https://github.com/jkcclemens/gitfreq
cd gitfreq
```

Now, you can **either** update your `PATH` and run `cargo install`, or you can do the following:

```sh
cargo build --release
cd target/release
strip git-freq # optional. usage varies by system
cp git-freq /usr/local/bin/
```

## Usage

    git freq [reference_date] [days]

`reference_date` is a date in the `YYYY-mm-dd` format, which is the date on which `days_since` acts.

`days_since` is the amount of days relative to `reference_date` to look for commits.

So, `git freq 2000-01-30 -5` would look for commits between 2000-01-25 and 2000-01-30.

`git freq 2000-01-25 5` would look for commits between 2000-01-25 and 2000-01-30, as well.

## Warning

This is the third language I've rewritten this program in, and two of which (including this) have been learning
projects.

I wrote this to begin learning Rust. I will improve it over time, as I improve in the language.
