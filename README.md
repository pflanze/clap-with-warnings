# Derive macro for Clap to retain unused field warnings

The use of this crate is to re-enable compiler warnings on structs to
which the `clap::Parser` derive macro is added, so that when
forgetting to make use of a field, the compiler warning tells you
about it.

The longer explanation: Clap generates code with a method that updates
the struct fields, which sets the fields one by one. These writes
disable the Rust compiler warnings on fields that are never
read--sadly, apparently writes satisfy the reading requirement. This
means that the compiler does not warn about fields that are never read
by your code--from its perspective, they *are* used.

I don't see how to change Clap to solve this. I would think that this
is an issue that should be changed in the compiler, either by not
making writes silence the read warning, or by offering an attribute
that makes it skip particular writes, or perhaps by skipping them
automatically while in a trait implementation that has
`#[automatically_derived]` on it (which Clap does add).

Filling in the same fields when constructing structs does not silence
the warning. Hence a solution is to generate two structs. The Clap
derives are implemented on the first struct. After generating that
struct (parsing the command line arguments), the fields are copied
into the second struct, which is identical except for not having any
code from Clap on it--i.e. there are no writes to any of its fields. A
`From` implementation or similar will convert from the first to the
second. Then the second can be used in the code, and unused warnings
will work.
