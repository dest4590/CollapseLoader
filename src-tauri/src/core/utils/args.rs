use std::env;

pub struct Args {
    pub backend_fix: bool,
}

impl Args {
    pub fn parse() -> Self {
        let args: Vec<String> = env::args().collect();

        if args.iter().any(|a| a == "--help" || a == "-h") {
            Self::print_help();
            std::process::exit(0);
        }

        let backend_fix = args.iter().any(|a| a == "--backend-fix");

        Self { backend_fix }
    }

    fn print_help() {
        println!("CollapseLoader v{}", env!("CARGO_PKG_VERSION"));
        println!("Usage: collapseloader [OPTIONS]");
        println!();
        println!("Options:");
        println!("  --backend-fix    Force use x11 GDK backend (Linux only)");
        println!("  --help, -h       Print help information");
    }

    pub fn process(&self) {
        #[cfg(target_os = "linux")]
        {
            if env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
                env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }

            if self.backend_fix && env::var_os("GDK_BACKEND").is_none() {
                log_info!("Applying backend fix: setting GDK_BACKEND=x11");
                env::set_var("GDK_BACKEND", "x11");
            }
        }
    }
}
