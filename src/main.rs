extern crate serde_json;
extern crate silkthread_base;
extern crate silkthread_toolkit;
extern crate silkthread_lang;

use serde_json::*;

use silkthread_base::*;
use silkthread_base::basic::*;
use silkthread_toolkit::io::*;
use silkthread_toolkit::io::tool::*;
use silkthread_lang::script::*;
use silkthread_lang::script::tool::*;

fn main() {
    // Start up
    let main_env = DynamicEnvironment::new();
    main_env.import(IoTools::new_stdio());
    main_env.import(ScriptTools::new());

    // Display a header
    let print_string = main_env.get_typed_tool::<String, ()>("print").unwrap();
    print_string.invoke(format!("{} {} by {}\n", env!("CARGO_PKG_NAME"),  env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS")), &main_env).unwrap();

    // Start a REPL
    loop {
        let print_string    = main_env.get_typed_tool::<String, ()>(PRINT).unwrap();
        let print_value     = main_env.get_typed_tool::<Value, ()>(PRINT).unwrap();
        let read_line       = main_env.get_typed_tool::<(), ReadLineResult>(READ_LINE).unwrap();
        let lex_line        = main_env.get_typed_tool::<String, Value>(LEX_SCRIPT).unwrap();
        let parse_script    = main_env.get_json_tool(PARSE_SCRIPT).unwrap();
        let eval_script     = main_env.get_json_tool(EVAL_SCRIPT).unwrap();
        let display_prompt  = main_env.get_typed_tool::<(), ()>("display-prompt");

        // Display a prompt
        display_prompt
            .map(|tool| tool.invoke((), &main_env).unwrap())
            .map_err(|_| print_string.invoke(String::from("\n=» "), &main_env).unwrap())
            .unwrap_or(());

        // Read the next line
        let next_line = read_line.invoke((), &main_env);

        match next_line {
            Ok(result) => {
                // Process the line
                // TODO!
                let lexed       = lex_line.invoke(result.line, &main_env).unwrap();
                let parsed      = parse_script.invoke_json(lexed, &main_env).unwrap();
                let eval_result = eval_script.invoke_json(parsed, &main_env);
                print_value.invoke(eval_result.unwrap(), &main_env).unwrap();
                print_string.invoke(String::from("\n"), &main_env).unwrap();

                // Stop on EOF
                if result.eof {
                    break;
                }
            },

            Err(erm) => {
                // Stop if we hit an error
                print_string.invoke(String::from("Error while reading from prompt: "), &main_env).unwrap();
                print_value.invoke(erm, &main_env).unwrap();
                break;
            },
        }
    }
}
