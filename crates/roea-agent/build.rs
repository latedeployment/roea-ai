fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Declare custom cfg flag so the compiler knows about it
    println!("cargo::rustc-check-cfg=cfg(ebpf_available)");

    // Compile protobuf definitions
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["../../proto/roea.proto"], &["../../proto"])?;

    // On Linux, compile eBPF programs
    #[cfg(target_os = "linux")]
    {
        compile_bpf()?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn compile_bpf() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;

    let src_dir = PathBuf::from("src/bpf");
    let bpf_source = src_dir.join("process_monitor.bpf.c");

    // Only compile if the BPF source exists
    if !bpf_source.exists() {
        println!("cargo:warning=BPF source not found at {:?}, skipping eBPF compilation", bpf_source);
        return Ok(());
    }

    // Check if vmlinux.h exists, skip if not (user needs to generate it)
    let vmlinux_path = src_dir.join("vmlinux.h");
    if !vmlinux_path.exists() {
        println!("cargo:warning=vmlinux.h not found at {:?}", vmlinux_path);
        println!("cargo:warning=To enable eBPF monitoring, generate vmlinux.h with:");
        println!("cargo:warning=  bpftool btf dump file /sys/kernel/btf/vmlinux format c > src/bpf/vmlinux.h");
        println!("cargo:warning=Skipping eBPF compilation for now");
        return Ok(());
    }

    println!("cargo:rerun-if-changed=src/bpf/process_monitor.bpf.c");
    println!("cargo:rerun-if-changed=src/bpf/vmlinux.h");

    // Use libbpf-cargo to compile BPF programs
    libbpf_cargo::SkeletonBuilder::new()
        .source(&bpf_source)
        .clang_args([
            "-I", src_dir.to_str().unwrap(),
            "-Wno-unused-value",
            "-Wno-pointer-sign",
            "-Wno-compare-distinct-pointer-types",
            "-Wextra",
            "-g",
            "-O2",
        ])
        .build_and_generate(
            &PathBuf::from(std::env::var("OUT_DIR")?).join("process_monitor.skel.rs")
        )?;

    // Set cfg flag to enable eBPF module compilation
    println!("cargo::rustc-cfg=ebpf_available");
    println!("cargo:warning=eBPF program compiled successfully");

    Ok(())
}
