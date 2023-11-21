# Using

## Transformations

- lowercase - convert to lowercase
- uppercase - convert to uppercase
- no-spaces - remove spaces
- slugify - convert unicode to ascii slug
- reverse - reverse string
- sha256 - sha256 hash of string
- csv - print table from .csv file

## Running application

- `cargo run`
    - enters interactive mode
    - as CLI input you have to first specify transformation and then string
    - there is endless loop of insterting this inputs
    - example: `uppercase Rust`
        - result: `RUST`
- `cargo run <transformation>`
    - `<transformation>` must be one of possible transformations
    - as CLI input you have to specify only string
    - example: `Rust` with `<transformation>` = `uppercase`
        - result: `RUST`
- `cargo run <transformation> <string>`
    - `<transformation>` must be one of possible transformations
    - `<string>` is text that will be converted
    - example: `cargo run uppercase Rust`
        - result: `RUST`
