pub fn get_template(template: Option<String>) -> &'static [u8] {
    match template {
        None => include_bytes!("templates/default.toml"),
        Some(x) => match x.to_lowercase().as_str() {
            "rust"          => include_bytes!("templates/rust.toml"),
            "node" | "nodejs" => include_bytes!("templates/node.toml"),
            "go"            => include_bytes!("templates/go.toml"),
            "c"             => include_bytes!("templates/c.toml"),
            "cpp"           => include_bytes!("templates/cpp.toml"),
            "ruby"          => include_bytes!("templates/ruby.toml"),
            "php"           => include_bytes!("templates/php.toml"),
            "java"          => include_bytes!("templates/java.toml"),
            "kotlin"        => include_bytes!("templates/kotlin.toml"),
            "swift"         => include_bytes!("templates/swift.toml"),
            "zig"           => include_bytes!("templates/zig.toml"),
            "elixir"        => include_bytes!("templates/elixir.toml"),
            "haskell"       => include_bytes!("templates/haskell.toml"),
            "css" | "scss"  => include_bytes!("templates/css.toml"),
            "lua"           => include_bytes!("templates/lua.toml"),
            "shell" | "sh"  => include_bytes!("templates/shell.toml"),
            _               => include_bytes!("templates/default.toml"),
        },
    }
}