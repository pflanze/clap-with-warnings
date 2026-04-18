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
