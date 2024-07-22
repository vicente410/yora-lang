# Yora
Yora is a multi-paradigm programming language created with the main purpose of learning more about compiler and language design. It is, at least for now, a recreational project. The Yora development team is fully open to ideas and thoughts on all features, all feedback is more than welcome!

## Design Goals
All development decisions are taken with each of these goals in mind. Yora should aim to be:
- **Performant**: Both compiling and running should be fast and memory efficient, allowing for high performance applications.
- **Comfortable**: Expressive and yet easy to use, Yora should allow for clean, readable and extensible code.
- **Multi-paradigm**: Providing the tools to solve every problem in the most adequate way, with features from the procedural, functional and object-oriented paradigms.
- **Fun**: Above all, this is a passion project, so it should be enjoyable to use and develop.

## Development
Yora is currently in very early development and new features are released very frequently. Before the release of 1.0.0, stability and backwards compatibility are not the main concern, to allow for rapid development. The main feature in 0.2.0 is expected to be functions. As of now Yora is being interpreted instead of compiled to allow for quicker development of the language itself.

## Installation
The Yora compiler will later be rewritten in Yora itself, but for now the first compiler is written in Rust. As such, clone this repo and run ```cargo install --path /path/to/yora-compiler```. Use ```yora filename.yr``` to run your code. For now, binaries published with a release only support linux.
