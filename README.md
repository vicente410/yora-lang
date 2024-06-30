# Yora
Yora is a multi-paradigm programming language created with the main purpose of learning more about compiler and language design. It is, at least for now, a recreational project. The Yora development team is fully open to ideas and thoughts on all features, all feedback is more than welcome!

## Design Goals
All development decisions are taken with each of these goals in mind. Yora should aim to be:
- **Performant**: Both compiling and running should be fast and memory efficient, allowing for high performance applications.
- **Comfortable**: Expressive and yet easy to use, Yora should allow for clean, readable and extensible code.
- **Multi-paradigm**: Providing the tools to solve every problem in the most adequate way, with features from various paradigms.
- **Fun**: Above all, this is a passion project, so it should be enjoyable to use and develop.

## Procedural
Being closer to the machine allows us to be more direct with our code and have a closer relationship with the computer. Computers are inherently procedural, so by directly manipulating memory with allocators and by using procedures, Yora can more directly correlate with machine instructions. This allows for increased efficiency of the produced code.

## Functional
Yora is strongly inspired by functional programming languages. While state and variables are allowed for ease of use, Yora has various features from the functional world, allowing development of pure functional code if so desired. Some of these features include: first-class and high-order functions, fully expressive syntax, algebraic data types and anonymous functions. These allow us to write clean and expressive code, using tools like filters and mapping. On the other hand, algebraic data types provide a rich type system that is further enhanced with traits.

## Object-Oriented
Aside from being known for their verbosity, object-oriented languages have multiple great ideas such as encouraging encapsulation, abstraction of data and modeling problems in a many-to-many relationship with objects. These principles allow us to decouple our programs for even more modularity, easily refactor code and API creation. Yora implements these features with module visibility and generalized methods for all types. Furthermore, traits work in a similar fashion to interfaces, defining a set of functions to be implemented by any type.

## Development
Yora is currently in very early development, aiming to release 0.1.0 in the near future. The first release will include the basic features of the language: variables, an integer and a boolean type, control-flow and basic compiler errors. It will also be the basic framework from which we will build upon all of the mentioned features. After that, Yora will continue its rapid development. Before the release of 1.0.0, stability and backwards compatibility are not the main concern, to allow for rapid development. The main feature in 0.2.0 is expected to be functions.

## Installation
The Yora compiler will later be rewritten in Yora itself, but for now the first compiler is written in Rust. As such, clone this repo and run ```cargo install --path /path/to/yora-compiler```. You can then run ```yora-compiler -h``` for more information.
