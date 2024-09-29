use camino::Utf8Path;
use tokio::sync::OnceCell;

mod flatten;
mod output;

pub static TAILWIND: Source<output::File, flatten::None> = Source {
    name: "tailwind",
    url: "https://github.com/tailwindlabs/tailwindcss/releases/download/v3.4.4/{TARGET}",
    cell: OnceCell::const_new(),
    target: |arch, os| match (arch, os) {
        ("x86_64", "macos") => Some("tailwindcss-macos-x64"),
        _ => None,
    },
    output: output::File { mode: 0o755 },
    flatten: flatten::None,
};

pub static ESBUILD: Source<output::TarGz, flatten::Dir> = Source {
    name: "esbuild",
    url: "https://registry.npmjs.org/@esbuild/{TARGET}/-/{TARGET}-0.23.0.tgz",
    cell: OnceCell::const_new(),
    target: |arch, os| match (arch, os) {
        ("x86_64", "macos") => Some("darwin-x64"),
        _ => None,
    },
    output: output::TarGz,
    flatten: flatten::Dir("package/bin/esbuild"),
};

#[derive(Debug)]
pub struct Source<O, F> {
    name: &'static str,
    url: &'static str,
    cell: OnceCell<Box<Utf8Path>>,
    target: fn(&'static str, &'static str) -> Option<&'static str>,
    output: O,
    flatten: F,
}
