fn main() {
    println!("cargo:rerun-if-changed=../assets/web/front.ico");

    #[cfg(windows)]
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_icon("../assets/web/front.ico");
        resource
            .compile()
            .expect("no se pudo embebir el icono de Packwire en el ejecutable Windows");
    }
}
