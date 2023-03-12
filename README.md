# Rust Experiments
This repository contains a series of experimental projects to learn more about the Rust programming language. The majority of these projects are from 2018, with **Cross** a 2023 project to explore the language 5 years later.

## Basics
Basic experiments

## Fractal
Julia fractal rendering

## Voxelli
Voxel-based rendering
![Voxelli.PNG](./Voxelli.PNG "Rendering in 3D with Rust")

## Cross
(2023 IN PROGRESS) Cross-stitch pattern generator from arbitrary images.

# 2018 analysis
## Rust Advantages
- Easy to get packages for all sorts of functionality.
- No GC penalty for operations.
- Very convenient structures, primitives, and general syntax.
- Lots of compile type checking for memory usage and typing.
- Very good VS Code editor support. 

## Rust Disadvantages
- Rust separates out each source file as its own module. This means you need a lot of boilerplate code to link files together and you tend to write larger (and harded to comprehend) files as a result. This is better than C++ header files, though.
- Build times are comparable to C++ or slower for small changes. 
- Same complexities with C++ with regards to Unicode string manipulation and package diamond dependency complexities.
- External code tends to make heavy use of macros, which makes that code harder to understand.
- Manual memory management comes with extra developer-time overhead. 