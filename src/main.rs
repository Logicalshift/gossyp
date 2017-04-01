extern crate serde_json;
extern crate gossyp_base;
extern crate gossyp_toolkit;
extern crate gossyp_lang;

use serde_json::*;

use gossyp_base::*;
use gossyp_base::basic::*;
use gossyp_toolkit::io::*;
use gossyp_toolkit::io::tool::*;
use gossyp_lang::script::*;
use gossyp_lang::script::tool::*;

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
                // Evaluate the result
                let eval_result = lex_line.invoke(result.line, &main_env)
                    .and_then(|lexed| parse_script.invoke_json(lexed, &main_env))
                    .and_then(|parsed| eval_script.invoke_json(parsed, &main_env));

                // Print it out
                match eval_result {
                    Ok(Value::Null) => { },
                    Ok(not_null)    => { print_value.invoke(not_null, &main_env).unwrap(); },
                    Err(erm)        => {
                        print_string.invoke(String::from("*** Error: "), &main_env).unwrap();
                        print_value.invoke(erm, &main_env).unwrap();
                    }
                }

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
