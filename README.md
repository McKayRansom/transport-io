
# Transport IO Game

Transport-IO is a transportation simulation game. Build road networks, railways, canals, and airports across a map with diverse terrain including cities, mountains, rivers, and oceans. Route busses, metros, ferries, and flights to deliver people and cargo to dynamically generated destinations.

Try it live at -> https://mckayransom.github.io/transport-io/

## Alpha target features
The transport-IO 0.1 alpha focuses on building road networks:
 - [ ] Build roads with realistic traffic modeling, including acceleration, route planning, and congestion.
 - [x] Create GOATED intersections using ROUNDABOUTS™ 
 - [ ] Increase throughput with highway features including bridges and tunnels
## Full game target features
- [ ] Transport networks with intuitive visual route creation
- [ ] City growth is based on how well they are serviced; larger cities will require more goods and connections to grow! Unserviced houses will create GODAWEFULL CARS that will gum up the road network!
- [ ] Unlock new features and learn how to play as you progress through the tutorials
- [ ] Freeplay on procedurally generated map
- [ ] Rail networks with automatic signaling, bidirectional track, and diagonal curves.
- [ ] Ferry networks with GIGASIZE capacity on oceans, lakes, rivers, and canals.
- [ ] Custom airports: runways, taxiways, gates, and terminals to customize your airport layout.
- [ ] Connect to friend’s levels for even larger city sizes! (Not multiplayer just a share maps feature)

Game design guidelines:
 - The fun is the network! IMO these types of games are about building and upgrading transport networks. Side features like Combat, Signal managment, Vehicle managment, etc... aren't interesting.
   - Half 1: Build a network and design intersections/best layouts of things if you want
   - Half 2: Route services on the network 
 - Keep it Simple: Every feature should enhance the gameplay, No duplicate features or busy work!
 - Do it automatically: Don't make the user manually do things that could easily be done automatically

Inspiration:
 - OpenTTD is the game I want hidden under a dated UI and overcomplicated vehicle and route management.
 - Factorio is incredible for quality-of-life features, but is overly complicated and the only networks are train networks which are not the focus. 
 - Mini Metro and Mini Motorways show that you can build an interesting network with much simpler mechanics, but the short levels limit network size.

Programming guidelines:
 - Try to follow rust standards, and avoid panics to get the benefits of rust. Handle all errors, no unwrap() in non-test code
 - Unit Tests as much as it makes sense
 - KISS!

## Developing
Transport IO is written in Rust using the https://macroquad.rs/ library to build for any platform including native, mobile, and webassembly

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

Highly recommend "bacon" to automatically run tests on every save for quick development cycle:

    cargo install --locked bacon

    bacon test

