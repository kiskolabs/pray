use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let count: usize = arguments
        .next()
        .ok_or("usage: destinations COUNT PROGRAM [ARG...]")?
        .parse()?;
    if count == 0 || count > 10_000 {
        return Err("COUNT must be between 1 and 10000".into());
    }
    let oversized = std::env::var_os("PRAY_BENCH_OVERSIZED").is_some();
    let program = arguments.next().ok_or("missing CLI program")?;
    let root = tempfile::tempdir()?;
    let root = root.path();
    fs::create_dir_all(root.join("package/files"))?;
    fs::create_dir_all(root.join("out/files"))?;
    let mut names = Vec::new();
    let content = b"fixture content\n";
    for index in 0..count {
        let name = format!("files/{index:05}.txt");
        fs::write(root.join("package").join(&name), content)?;
        fs::write(root.join("out").join(&name), content)?;
        if oversized {
            fs::OpenOptions::new()
                .write(true)
                .open(root.join("out").join(&name))?
                .set_len(32 * 1024 * 1024 + 1)?;
        }
        names.push(name);
    }
    fs::write(
        root.join("Prayfile"),
        "prayfile \"1\"\ntree \"out\" do\n pray \"sample/files\", path: \"package\"\nend\n",
    )?;
    fs::write(root.join("package/files.prayspec"), format!("Package::Specification.new do |spec|\n spec.name = \"sample/files\"\n spec.version = \"1.0.0\"\n spec.files = [{}]\n spec.exports = {{ \"files\" => {{ type: \"folder\", path: \"files\" }} }}\nend\n",
        names.iter().map(|name| format!("\"{name}\"")).collect::<Vec<_>>().join(", ")))?;
    let mut lockfile = fs::File::create(root.join("Prayfile.lock"))?;
    writeln!(lockfile, "prayfile_lock = \"1\"\nspec = \"prayfile-1\"\ngenerated_by = \"pray benchmark\"\nmanifest_hash = \"fixture\"\nsource = []\npackage = []\ntarget = []\nmanaged_span = []")?;
    for name in names {
        writeln!(lockfile, "\n[[provisioned]]\npath = \"out/{name}\"\ncontent_hash = \"{}\"\npackage = \"sample/files\"\nexport = \"files\"", pray_core::hashing::sha256_prefixed(content))?;
    }
    eprintln!("destination-plan leaves={count} oversized={oversized}");
    let mut command = Command::new("/usr/bin/time");
    command.arg(if cfg!(target_os = "macos") {
        "-l"
    } else {
        "-v"
    });
    command.arg(program).args(arguments).args([
        "--path",
        root.to_str().ok_or("invalid temporary path")?,
        "plan",
    ]);
    command
        .env("PRAY_HOME", root.join("home"))
        .env("PRAY_CACHE", root.join("cache"));
    command.env_remove("PRAY_FILE_PATH").env_remove("PRAY_ENV");
    let status = command.stdout(Stdio::null()).status()?;
    if !(status.success() && !oversized || oversized && status.code() == Some(5)) {
        return Err(format!("CLI exited {status}").into());
    }
    Ok(())
}
