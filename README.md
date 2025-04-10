This is a collection of types, macros, and functions that we personally found
useful and nice to have.

One of the most useful aspects of what I have made surrounds the topic
of getting input from the terminal. To understand how to use them,
I recommend reading the documentation.

If you want to use my libray, you will have to add the dependency
to Cargo.toml directly because this is not on crates.io
One option is to add
```toml
albatrice = { git = "https://github.com/albatrice/albatrice.git", branch = "release" }
```
The release branch will always give you the latest release, which
means that it will be stable. However, if you want a specific release,
you can use
```toml
albatrice = { git = "https://github.com/albatrice/albatrice.git", tag = "v0.1.1" }
```
Which will get you the v0.1.1 release. If you want to have the most
experimental version(which will be a great help to me for finding bugs),
you can remove the version specification and pull from the branch I actively
work on, however, it is not stable and everything is subject to change at any
time.
```toml
albatrice = { git = "https://github.com/albatrice/albatrice.git" }
```
If you find a bug while using this, please report it as an issue,
and if you find a significant security vulnerability(although
I don't have anything in this library yet that would warrant that)
please report it privately through GitHub.

# Binaries
## file_comm
file_comm is a binary for sending files through direct peer to peer with a format
handshake while allowing either the sender or receiver to host the
connection.
To install it, run
```
cargo install --git https://github.com/albatrice/albatrice.git" --branch release
```
Once you have done that, you will be able to get more information by using
```
file_comm help
```
Or, if you have downloaded this library in its entirety for whatever
reason, you can get more information through
```
cargo run -- help
```
