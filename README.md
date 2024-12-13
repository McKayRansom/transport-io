
# Transport IO Game
Transport game insired by a mix of shapez.io, mini metro, mini motorways, OpenTTD and Factorio.
Currently Pre-alpha.

Development guidlines:
 - The fun is the network! IMO these types of games are about building and upgrading transport networks. Side features like Combat, Signal managment, Vehicle managment, etc... aren't interesting. 
 - I started this game because I love factorio but feel like it overcomplicated things, and I love OpenTTD but think it's a little to hard to build a big network. Mini Metro and Mini Motorways show that you can build an interesting network with much simpler mechanics, but they are missing the scaling aspect.
 - Keep it Simple: Every feature should enhance the gameplay, No duplicate features or busy work!
 - Do it automatically: Don't make the user manually do things that could easily be done automatically


## Building
Transport IO uses the https://macroquad.rs/ library to build for any platform including webassembly

### Native
    cargo run

### Webassembly

To build and pack website into deploy/ and start a website on localhost:4000

    ./deploy_local

## Testing

    cargo test

### Code coverage

    cargo install cargo-tarpaulin

    cargo tarpaulin 

To see code coverage gutters use "Coverage Gutters" VSCode extenstion and add VSCode settings:

    "coverage-gutters.coverageFileNames": [

        "cobertura.xml"
    ],

Then have tarpaulin output in cobertura xml format that "Coverage Gutters" can understand:

    cargo tarpaulin --out Xml


## Profiling

Samply seems to work pretty well https://github.com/mstange/samply

Try:

    cargo install --locked samply

    samply record target/debug/transport-io

## Live build

Highly recommend "bacon" to re-run tests when file changed for very fast feedback:

    cargo install --locked bacon

    bacon test

