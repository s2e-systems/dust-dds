# DustDDS: Software design document

## Introduction
Data Distribution Services (DDS) is a middleware protocol and API standard for data-centric connectivity. The main goal of DDS is to share the right data at the right place at the right time, even between time-decoupled publishers and consumers. DustDDS is the [S2E Software Systems](https://www.s2e-systems.com) implementation of the [Data Distribution Services (DDS)](https://www.omg.org/omg-dds-portal/) and [Real-time Publisher-Subscriber (RTPS)](https://www.omg.org/spec/DDSI-RTPS/About-DDSI-RTPS/) protocols using the [Rust programming language](https://www.rust-lang.org/).

This Software Design Document (SDD) outlines the architectural design, key design choices and trade-offs related to the development of DustDDS. This document is intended for those who are interested in the internal workings of DustDDS and to keep track of important design decisions during the development process. It is not meant as a documentation of actual functionality which is instead done in the source code itself using cargo docs. For more high-level information and usage examples please refer to the [README](../README.md).

## High-level architecture

The implementation of DustDDS is done in what could be better described as layers which implement different parts of the functionality. Looking at it from a DustDDS library user the layers can be described as follows:

1. __DDS API Layer__
The DDS API layer serves as the fundamental interface for DustDDS, implementing the specifications defined by the Object Management Group (OMG) for the Data Distribution Service (DDS). This layer exposes the standardized DDS functionalities to users, allowing them to interact with DustDDS using well-defined DDS concepts and operations. It acts as the bridge between the user's application and the underlying data distribution infrastructure. Much of the functionality implemented in this layer revolves around communicating with the different actors.

2. __Actors__
The DustDDS implementation follows the actor model. In short, each class is typically wrapped into an actor object and the methods are invoked by sending a message using a channel.  Actors may modify their own private state, but can only affect each other indirectly through messaging (removing the need for lock-based synchronization). More information on actor can be found on [Wikipedia](https://en.wikipedia.org/wiki/Actor_model) or [on this blog article focusing on Rust](https://ryhl.io/blog/actors-with-tokio/).

3. __RTPS__
The RTPS (Real-time Publish-Subscribe) layer implements the Object Management Group's RTPS standard. This layer includes types, traits, and functionality that allow using the RTPS protocol. This is needed for interoperable data exchange in real-time and distributed systems.

## Implementation details

In this section we describe the design decision and some trade-offs and technical limitations analysis. For DustDDS we are using only Stable Rust with unsafe code explicitly forbidden. At the time of writting we are using Rust edition 2021 with compiler version 1.73.0.

### DDS API layer

The DDS API Layer in DustDDS adheres closely to the Object Management Group (OMG) DDS standard, which specifies DDS API methods as interfaces. In Rust, the natural mapping for such IDL interfaces would be traits. However, due to certain technical limitations, DustDDS implements the DDS API methods as inherent functions of objects. This choice stems from two key limitations:

1. __Returning Opaque Types in Traits__
Opaque types are types whose details are hidden from the user and are only accessible through a defined set of functions or methods. They do not require dynamic dispatch and it would therefore be a good choice for this type of API. Lets consider the following simplified FooDataWriter trait

```rust
trait FooDataWriter<Foo> {
    fn write(&self, data: &Foo) -> Result<(),()>;
}

```

The DDS standard defines that a Publisher can create Data Writers for any different *Foo* type for which data can be distributed using the middleware. This creation in a simplified form looks something like this:

```rust
trait Publisher {
    fn create_datawriter<Foo>(&self) -> impl FooDataWriter<Foo>;
}
```

This however fails to compile with `error[E0562]: impl Trait only allowed in function and inherent method return types, not in trait method return types`. An alternative to this could be to use an associated type:

```rust
trait Publisher {
    type DataWriter<Foo>;
    fn create_datawriter<Foo>(&self) -> Self::DataWriter<Foo>;
}

impl Publisher for PublisherNode {
    type DataWriter<Foo> = impl FooDataWriter<Foo>;
    fn create_datawriter<Foo>(&self) -> Self::DataWriter<Foo> {
        todo!()
    }
}
```

However, having opaque types return from associated types is still unstable `error[E0658]: impl Trait in associated types is unstable`.

2. __Trait objects with generic methods__
The API could instead return trait objects and do dynamic dispatch which for the usages of DustDDS would likely have an irrelevant impact. The publisher trait could then be modified to:

```rust
trait Publisher {
    fn create_datawriter<Foo>(&self) -> Box<dyn FooDataWriter<Foo>>;
}
```

The following level is the *Domain Participant* which creates a *Publisher* in the following simplified manner:

```rust
trait DomainParticipant {
    fn create_publisher(&self) -> Box<dyn Publisher>;
}
```

This results in the error `error[E0038]: the trait Publisher cannot be made into an object. ... note: for a trait to be "object safe" it needs to allow building a vtable to allow the call to be resolvable dynamically;`

Taking into account these constraints, at this point the decision is either to have a struct implementing the DDS API methods directly on have the trait return a mix of concrete types and trait objects.