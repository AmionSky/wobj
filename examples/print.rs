use std::collections::HashSet;
use std::path::Path;

fn main() {
    let path = Path::new("res/test.obj");
    let obj_file = std::fs::read(path).unwrap();
    let obj = wobj::Obj::parse(&obj_file);

    // Print OBJ and collect all MTLs
    let mut mtllibs = HashSet::new();
    match obj {
        Ok(obj) => {
            println!("OBJ: {obj:#?}");
            for object in obj.objects() {
                if let Some(mtllib) = object.mtllib() {
                    mtllibs.insert(mtllib.clone());
                }
            }
        }
        Err(error) => eprintln!("OBJ: {error}"),
    }

    // Print MTLs
    let parent = path.parent().unwrap();
    for mtllib in mtllibs {
        let mtl_file = std::fs::read(parent.join(mtllib)).unwrap();
        let mtl = wobj::parse_mtl(&mtl_file);
        match mtl {
            Ok(mtl) => println!("MTL: {mtl:#?}"),
            Err(error) => eprintln!("MTL: {error}"),
        }
    }
}
