fn main() {
    println!("cargo:rerun-if-changed=include/src/font_renderer.cpp");
    println!("cargo:rerun-if-changed=include/src/glad.c");
    println!("cargo:rerun-if-changed=include/src/GLDebug.cpp");
    println!("cargo:rerun-if-changed=include/src/shader.cpp");
    println!("cargo:rerun-if-changed=include/src/sprites.cpp");
    println!("cargo:rerun-if-changed=include/src/sprites_single.cpp");
    println!("cargo:rerun-if-changed=include/src/stb_image.c");
    println!("cargo:rerun-if-changed=include/src/initializer.cpp");
    

    cc::Build::new()
        .cpp(true)
        .flag("-std=c++17")
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-pthread")
        .include("include/include")
        .include("/usr/include/freetype2")
        .include("/usr/include/libpng16")
        .include("/usr/include/harfbuzz")
        .include("/usr/include/glib-2.0")
        .include("/usr/lib/glib-2.0/include")
        .include("/usr/include/sysprof-6")
        .files(["include/src/font_renderer.cpp",
        "include/src/glad.c",
        "include/src/GLDebug.cpp",
        "include/src/shader.cpp",
        "include/src/sprites.cpp",
        "include/src/sprites_single.cpp",
        "include/src/stb_image.c",
        "include/src/initializer.cpp"])
        .compile("rendering");

    println!("cargo:rustc-link-lib=GL");
    println!("cargo:rustc-link-lib=glfw");
    println!("cargo:rustc-link-lib=freetype");
}
