// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;
use std::process::Command;

use build::{Build, Compiler};

pub fn linkcheck(build: &Build, stage: u32, host: &str) {
    println!("Linkcheck stage{} ({})", stage, host);
    let compiler = Compiler::new(stage, host);
    build.run(build.tool_cmd(&compiler, "linkchecker")
                   .arg(build.out.join(host).join("doc")));
}

pub fn cargotest(build: &Build, stage: u32, host: &str) {

    let ref compiler = Compiler::new(stage, host);

    // Configure PATH to find the right rustc. NB. we have to use PATH
    // and not RUSTC because the Cargo test suite has tests that will
    // fail if rustc is not spelled `rustc`.
    let path = build.sysroot(compiler).join("bin");
    let old_path = ::std::env::var("PATH").expect("");
    let sep = if cfg!(windows) { ";" } else {":" };
    let ref newpath = format!("{}{}{}", path.display(), sep, old_path);

    build.run(build.tool_cmd(compiler, "cargotest")
              .env("PATH", newpath)
              .arg(&build.cargo));
}

fn testdir(build: &Build, host: &str) -> PathBuf {
    build.out.join(host).join("test")
}

pub fn compiletest(build: &Build,
                   compiler: &Compiler,
                   target: &str,
                   mode: &str,
                   suite: &str) {
    let compiletest = build.tool(compiler, "compiletest");
    let mut cmd = Command::new(&compiletest);

    cmd.arg("--compile-lib-path").arg(build.rustc_libdir(compiler));
    cmd.arg("--run-lib-path").arg(build.sysroot_libdir(compiler, target));
    cmd.arg("--rustc-path").arg(build.compiler_path(compiler));
    cmd.arg("--rustdoc-path").arg(build.rustdoc(compiler));
    cmd.arg("--src-base").arg(build.src.join("src/test").join(suite));
    cmd.arg("--aux-base").arg(build.src.join("src/test/auxiliary"));
    cmd.arg("--build-base").arg(testdir(build, compiler.host).join(suite));
    cmd.arg("--stage-id").arg(format!("stage{}-{}", compiler.stage, target));
    cmd.arg("--mode").arg(mode);
    cmd.arg("--target").arg(target);
    cmd.arg("--host").arg(compiler.host);

    let filecheck = build.llvm_filecheck(&build.config.build);
    cmd.arg("--llvm-bin-path").arg(filecheck.parent().unwrap());

    let linkflag = format!("-Lnative={}", build.test_helpers_out(target).display());
    cmd.arg("--host-rustcflags").arg("-Crpath");
    cmd.arg("--target-rustcflags").arg(format!("-Crpath {}", linkflag));

    // FIXME: needs android support
    cmd.arg("--android-cross-path").arg("");
    // FIXME: CFG_PYTHON should probably be detected more robustly elsewhere
    cmd.arg("--python").arg("python");

    cmd.args(&build.flags.args);

    if build.config.verbose || build.flags.verbose {
        cmd.arg("--verbose");
    }

    build.run(&mut cmd);
}
