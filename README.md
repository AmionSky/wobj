# wobj

Wavefront OBJ polygonal geometry and MTL material parser library.

- OBJ parsing only supports polygonal geometry and ignores all other statements.

- MTL parsing supports both the original spec and the PBR extensions.

## Usage

To load an OBJ file:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("assets/cube.obj")?;
    let obj = wobj::Obj::parse(&bytes)?;

    // To get the meshes
    for mesh in &obj.meshes() {
        println!("Name: {:?}", mesh.name());

        // To triangulate the meshes
        let (indicies, verticies) = mesh.triangulate()?;
    }

    Ok(())
}
```

To load a material:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("assets/cube.mtl")?;
    let mtl = wobj::Mtl::parse(&bytes)?;
    let material: Option<&wobj::Material> = mtl.get("CubeMaterial");
    Ok(())
}
```

Since OBJ files can use a different MTL file for each material, if we want to
load all used materials without wasting work:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let obj_path = std::path::Path::new("assets/cube.obj");
    let parent_path = obj_path.parent().unwrap();

    // Load the OBJ
    let obj = wobj::Obj::parse(&std::fs::read(obj_path)?)?;
    // Use a HashMap to store loaded MTLs
    let mut mtls = std::collections::HashMap::new();

    // Iterate through all meshes in the OBJ
    for mesh in &obj.meshes() {
        // Check if the mesh has a material
        if let Some(mtllib) = mesh.mtllib()
            && let Some(material_name) = mesh.material()
        {
            // Load the MTL if it has not been loaded before
            if !mtls.contains_key(mtllib) {
                // MTL-lib path is relative to the OBJ file
                let mtl_path = parent_path.join(mtllib);
                let mtl = wobj::Mtl::parse(&std::fs::read(mtl_path)?)?;
                mtls.insert(mtllib, mtl);
            }

            // Get the material
            let material = mtls
                .get(mtllib)
                .and_then(|m| m.get(material_name))
                .expect("Material not found");

            // Use the material
            println!("Material '{material_name}': {material:?}");
        }
    }

    Ok(())
}
```

## License

Licensed under either of

- MIT License ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)

at your option.
