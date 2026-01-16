use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(std::env::args_os().nth(1).expect("No path was specified!"));

    let obj_file = std::fs::read(&path)?;
    let now = Instant::now();
    let obj = wobj::Obj::parse(&obj_file)?;
    let load_time = now.elapsed();

    println!("OBJ: ({})", path.display());
    println!("  Parsed in {} seconds", load_time.as_secs_f64());
    println!("  Vertices:  {}", obj.vertices().len());
    println!("  Normals:  {}", obj.normals().len());
    println!("  UVs:  {}", obj.uvs().len());

    // Print OBJ object stats and collect MTL files
    println!("  Objects:");
    let mut mtl_files = HashSet::new();
    for mesh in obj.meshes() {
        println!(
            "    {}: material: {}, face count: {}",
            mesh.name().unwrap_or("<none>"),
            mesh.material().unwrap_or("<none>"),
            mesh.faces().len()
        );

        if let Some(mtllib) = mesh.mtllib() {
            mtl_files.insert(mtllib.to_path_buf());
        }
    }

    println!();

    let obj_dir = path.parent().expect("Path had no parent");
    for mtl_path in mtl_files {
        let file = std::fs::read(obj_dir.join(&mtl_path)).unwrap();

        let now = Instant::now();
        let mtl = wobj::Mtl::parse(&file)?;
        let load_time = now.elapsed();

        println!("MTL: ({})", mtl_path.display());
        println!("  Parsed in {} seconds", load_time.as_secs_f64());
        println!("  Material count: {}", mtl.inner().len());

        println!("  Materials:");
        for name in mtl.inner().keys() {
            println!("    {}", name);
        }

        println!();
    }

    #[cfg(feature = "trimesh")]
    {
        let now = Instant::now();
        let mut meshes = Vec::new();
        for mesh in obj.meshes() {
            meshes.push((
                mesh.name().unwrap_or("<none>").to_string(),
                mesh.triangulate().unwrap(),
            ));
        }
        let load_time = now.elapsed();

        println!("Triangulated meshes:");
        println!("  Generated in {} seconds", load_time.as_secs_f64());

        println!("  Meshes:");
        for (name, mesh) in meshes {
            println!(
                "    {}: indicies: {}, vertices: {}",
                name,
                mesh.0.0.len(),
                mesh.1.positions.len(),
            )
        }
    }

    Ok(())
}
