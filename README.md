# Attribute macro to retain unused field warnings when using Clap

This allows to re-enable compiler warnings on unused struct fields
when the derive macros from clap are used.

A longer explanation:

The Rust compiler warns when struct fields are not used. This "is
never read" warning stays even when structs are constructed (the field
is set at that point). But the warning goes away when fields are set
(e.g. in `foo.field = 1`). This is unfortunate, since the Clap derive
macros (at least `clap::Parser`) generate code that sets the field
this way (more precisely, via `&mut foo.field`, which also silences
the warning).

The macro offered by this crate generats two versions of the struct:
one with `WithoutWarnings` appended to its name, but with the clap
derives and everything else included. And a second struct with the
original name, but all traces of clap removed. It also adds a method
`with_warnings` on the `..WithoutWarnings` struct that converts it to
the struct with the original name (by destructuring the original then
constructing the new struct with the values), but also a `parse`
method on the struct with the original name which calls Clap's `parse`
and then `with_warnings`. This hack means that the struct with the
original name has no code generated for it that sets the fields, and
hence the "never read" warnings work.

## Example

The field `verbose` is never read in this example. But nonetheless,
there is no compiler warning.

```
use clap::Parser;

#[derive(clap::Parser, Debug)]
#[clap(name = "foo")]
struct Opt {
    /// Say what is being done
    #[clap(short, long)]
    verbose: bool,

    /// Be silent about some things
    #[clap(short, long)]
    quiet: bool,
}

fn main() {
    let opt = Opt::parse();
    println!("{opt:?}\nbe quiet?: {}", opt.quiet);
}
```

Now add the `#[clap_with_warnings]` attribute macro on top, and the
warning will appear.

```
use clap::Parser;
use clap_with_warnings::clap_with_warnings;

#[clap_with_warnings]
#[derive(clap::Parser, Debug)]
#[clap(name = "foo")]
struct Opt {
    /// Say what is being done
    #[clap(short, long)]
    verbose: bool,

    /// Be silent about some things
    #[clap(short, long)]
    quiet: bool,
}

fn main() {
    let opt = Opt::parse();
    println!("{opt:?}\nbe quiet?: {}", opt.quiet);
}
```
