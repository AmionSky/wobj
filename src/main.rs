use wobj::Obj;

fn main() {
    let obj = Obj::parse("/home/csanyi/Projects/bevy_obj/assets/cube.obj");
    println!("OBJ: {obj:?}");
}
