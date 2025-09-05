fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        // .out_dir(&out_dir)
        // .file_descriptor_set_path(&out_dir.join("descriptor.bin"))
        .compile_protos(
            &["../../api/id_generator/v1/id_generator.proto"],
            &["../../api/id_generator/v1", "../../api/third_party"],
        )?;

    tonic_prost_build::configure()
        // .out_dir(&out_dir)
        // .file_descriptor_set_path(&out_dir.join("descriptor.bin"))
        .compile_protos(
            &["../../api/user/v1/user.proto"],
            &["../../api/user/v1", "../../api/third_party"],
        )?;
    Ok(())
}
