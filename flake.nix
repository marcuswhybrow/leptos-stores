{
  description = "Rust language project using the Leptos Web Framework, Axum webserver, and Tailwind CSS styling";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Rust toolchain including rustc, cargo, and WASM build functionality
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Rust packaging for nix
    naersk.url = "github:nix-community/naersk";
  };

  outputs = inputs: let 
    pkgs = inputs.nixpkgs.legacyPackages.x86_64-linux;

    # Leptos benefits from Rust language features only available in the 
    # "nightly" version of Rust. Depending on a nightly build trades API 
    # stability for increased language functionality.
    toolchain = inputs.fenix.packages.x86_64-linux.combine [
      inputs.fenix.packages.x86_64-linux.latest.toolchain
      inputs.fenix.packages.x86_64-linux.targets.wasm32-unknown-unknown.latest.rust-std
    ];

    # https://github.com/nix-community/naersk
    naersk = pkgs.callPackage inputs.naersk {
      cargo = toolchain;
      rustc = toolchain;
    };

    cargo = (pkgs.lib.importTOML ./Cargo.toml).package;
  in {
    packages.x86_64-linux.my-app = naersk.buildPackage {
      name = cargo.name;
      version = cargo.version;
      src = pkgs.lib.cleanSource ./.;

      # Fixes production build 
      # 
      # Leptos' WASM URL is a function of Leptos' output name. This is set in 
      # Cargo.toml, which works fine whilst using the `cargo-leptos` dev server 
      # during development. Build for production using `nix build` or `nix run`
      # will (for some reason I don't understand) fail to pick up this variable 
      # from Cargo.toml, despite picking up other variables from Cargo.toml 
      # such as the server port number. 
      #
      # Naersk.buildPackage (I believe) sets up all unknown attributes as 
      # environment variables for the build process, which in this case tells
      # Leptos which output name to look for.
      LEPTOS_OUTPUT_NAME = cargo.metadata.leptos.output-name;
    };
    packages.x86_64-linux.default = inputs.self.packages.x86_64-linux.my-app;

    devShells.x86_64-linux.default = pkgs.mkShell {
      # Rust crate `rand` requires requires this for targetting wasm32-unknown-unknown.
      RUST_FLAGS = ''--cfg getrandom_backend="wasm_js"'';

      nativeBuildInputs = [ 
        # Language Server Protocal support for your editor
        pkgs.rust-analyzer

        # rustc, cargo, etc...
        toolchain

        # Optional rust error checker with a nice interface
        pkgs.bacon 

        # Hot reloading dev server for Leptos (+ more)
        pkgs.cargo-leptos

        # Leptos uses "Syntactically Awesome Style Sheets" instead of CSS for 
        # global styling
        pkgs.sass

        # This project has configured leptos to use Tailwind CSS for inline
        # styling of HTML elements using CSS classes, an CSS styling approach
        # pioneered by the Tailwind CSS project.
        pkgs.tailwindcss
      ];
    };
  };
}

