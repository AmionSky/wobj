use kiss3d::prelude::*;
use wobj::Obj;

#[kiss3d::main]
async fn main() {
    let mut window = Window::new("Kiss3d: cube").await;
    let mut camera = OrbitCamera3d::default();
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(0.0, 2.0, -2.0));

    let mut meshes = scene.add_group();
    render_obj(&mut meshes, "res/test.obj");

    // let mut c = scene.add_cube(1.0, 1.0, 1.0).set_color(RED);

    let rot = Quat::from_axis_angle(Vec3::Y, 0.014);

    while window.render_3d(&mut scene, &mut camera).await {
        meshes.rotate(rot);
    }
}

fn render_obj(scene: &mut SceneNode3d, path: &str) {
    /*let obj = Obj::parse(path).unwrap();

    let coords: Vec<_> = obj
        .vertecies()
        .iter()
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect();

    for object in obj.objects() {
        let faces = object
            .faces()
            .iter()
            .map(|f| [f[0] as u32, f[1] as u32, f[2] as u32])
            .collect();

        let mesh = GpuMesh3d::new(coords.clone(), faces, None, None, false);

        scene.add_mesh(Rc::new(RefCell::new(mesh)), Vec3::ONE);
    }*/
}
