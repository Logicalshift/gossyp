extern crate silkthread_base;
extern crate silkthread_toolkit;

use silkthread_base::basic::*;
use silkthread_toolkit::io::*;

fn main() {
    // Start up
    let main_env = DynamicEnvironment::new();
    main_env.import(IoTools::new_stdio());

    // Display header
    let print_string = main_env.get_typed_tool::<String, ()>("print").unwrap();
    print_string.invoke(format!("{} {} by {}\n", env!("CARGO_PKG_NAME"),  env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS")), &main_env).unwrap();
}
