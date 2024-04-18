fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::configure()
    //     .type_attribute("routeguide.Point", "#[derive(Hash)]")
    //     .compile(&["proto/routeguide/route_guide.proto"], &["proto"])
    //     .unwrap();

    tonic_build::compile_protos("proto/helloworld.proto")?;

    Ok(())
}
