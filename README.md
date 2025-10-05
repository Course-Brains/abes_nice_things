This is a collection of types, macros, and functions that I personally found
useful and nice to have.

One of the most useful aspects of what I have made surrounds the topic
of getting input from the terminal. To understand how to use them,
I recommend reading the documentation.

If you want to use my libray, you will have to add the dependency
to Cargo.toml directly because this is not on crates.io
One option is to add
```toml
abes_nice_things = { git = "https://github.com/Course-Brains/abes_nice_things.git", branch = "release" }
```
The release branch will always give you the latest release, which
means that it will be stable. However, if you want a specific release,
you can use
```toml
abes_nice_things = { git = "https://github.com/Course-Brains/abes_nice_things.git", tag = "v0.1.1" }
```
Which will get you the v0.1.1 release. If you want to have the most
experimental version(which will be a great help to me for finding bugs),
you can remove the version specification and pull from the branch I actively
work on, however, it is not stable and everything is subject to change at any
time.
```toml
abes_nice_things = { git = "https://github.com/Course-Brains/abes_nice_things.git" }
```
If you find a bug while using this, please report it as an issue,
and if you find a significant security vulnerability(although
I don't have anything in this library yet that would warrant that)
please report it privately through GitHub.

# Binaries
To install the binaries run
```
cargo install --git https://github.com/Course-Brains/abes_nice_things.git --branch release
```
## file\_comm
file\_comm is a binary for sending files through direct peer to peer with a format
handshake while allowing either the sender or receiver to host the
connection.

To see the specific documentation run
```
file_comm help
```

## run
This is a way to automate running projects in the correct way (although it is entirely
my own system and I would be surprised if it worked on something made by someone else)
which looks for a file in the current directory called `run` and tries
to run it, or if it is not there, it will see if there is a file
called `Cargo.toml` in which case it will run `cargo run --release`.

Notably, this will pass all arguments given to it forward onto
whatever it uses to run it, meaning that if there is a run script, it
will run the script with whatever arguments you gave to `run`. Or if
there is a `Cargo.toml` file then it will do `cargo run --release
[whatever arguments you gave]`

There is no built in documentation for this because if running `run
help` and catching that would block running actual projects with the
argument help which is why this details how it works.
