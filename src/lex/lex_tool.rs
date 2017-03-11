//!
//! The lex tool generates lexer tools from its input
//!

use std::result::Result;
use std::error::Error;
use std::iter::*;
use serde::*;
use serde_json::*;
use concordance::*;
use silkthread_base::*;
use silkthread_base::basic::*;
use silkthread_base::basic::tool_name::*;

///
/// Input for the lexer tool
///
#[derive(Serialize, Deserialize)]
pub struct LexToolInput {
    /// Name of the tool that the lexer will define
    pub new_tool_name:  String,

    /// The symbols that the lexer will match
    pub symbols:        Vec<LexToolSymbol>
}

///
/// Represents a lexer match
///
#[derive(Serialize, Deserialize)]
pub struct LexerMatch {
    /// Token that was matched
    pub token:      String,

    /// Phrase that was matched from the input
    pub matched:    String,

    /// Start of the match
    pub start:      i32,

    /// End of the match
    pub end:        i32
}

///
/// Lexer symbol
///
#[derive(Serialize, Deserialize)]
pub struct LexToolSymbol {
    /// The name of the symbol that will be generated if this match is made
    pub symbol_name:    String,

    /// The rule that will be matched against this symbol
    pub match_rule:     String
}

///
/// Lexer generation tool
///
pub struct LexTool {
}

impl LexTool {
    ///
    /// Creates a new lexer tool
    ///
    pub fn new() -> LexTool {
        LexTool { }
    }

    ///
    /// Converts a string containing a lexer regex into a concordance pattern
    ///
    pub fn pattern_for_string(regex: &str) -> Pattern<char> {
        // We'll process the regex as UTF-16 code points
        let regex_chars: Vec<char> = regex.chars().collect();

        // Go on to build the pattern
        LexTool::pattern_for_chars(&regex_chars)
    }

    ///
    /// Returns a substitute character for a character following a '\'
    ///
    fn special_character_char(c: char) -> char {
        match c {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            'w' => ' ',

            // Just the literal character if there's no match
            c => c
        }
    }

    ///
    /// Returns the pattern to use for a special character
    ///
    fn special_character_pattern(c: char) -> Pattern<char> {
        match c {
            // Any whitespace
            'w' => MatchAny(vec![
                Match(vec![' ']), 
                Match(vec!['\t']),
                Match(vec!['\n']), 
                Match(vec!['\r']), 
                Match(vec!['\u{0085}']), 
                Match(vec!['\u{00a0}']), 
                Match(vec!['\u{1680}']),
                MatchRange('\u{2000}', '\u{200a}'),
                Match(vec!['\u{2028}']),
                Match(vec!['\u{2029}']),
                Match(vec!['\u{202f}']),
                Match(vec!['\u{205f}']),
                Match(vec!['\u{3000}'])
            ]),

            // Just the literal character otherwise
            c => Match(vec![LexTool::special_character_char(c)])
        }
    }

    ///
    /// Finds a subpattern from the index of the '(' that starts it
    ///
    fn get_subpattern<'a>(regex: &'a [char], subpattern_start: usize) -> &'a [char] {
        let start_pos   = subpattern_start+1;
        let mut depth   = 1;
        let mut end_pos = start_pos;
        let regex_len   = regex.len();

        // Subpattern ends at the end of the regex or at the closing ')'
        while end_pos < regex_len && depth > 0 {
            let chr = regex[end_pos];

            match chr {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                '\\' => end_pos += 1,
                '[' => {
                    // '[)]' isn't a close bracket :-/
                    while end_pos < regex_len && regex[end_pos] != ']' {
                        if regex[end_pos] == '\\' {
                            end_pos += 1;
                        }
                        end_pos += 1;
                    }
                }

                _ => ()
            }

            end_pos += 1;
        }

        &regex[start_pos..end_pos]
    }

    ///
    /// Joins up any sequence of Match<x>, Match<y>
    ///
    fn join_matches(pattern: &mut Vec<Pattern<char>>) {
        let mut index = 1;

        while index < pattern.len() {
            let current = pattern[index].clone();

            if let Match(ref current) = current {
                let previous = pattern[index-1].clone();

                if let Match(ref previous) = previous {
                    // If we have two Matches one after the other, combine them into a phrase
                    // This and some other parts of this code can be improved by concatenating phrases all at once or by building into a new pattern array
                    let mut phrase = previous.clone();
                    phrase.extend(current);

                    pattern[index-1] = Match(phrase);
                    pattern.remove(index);

                    index -= 1;
                }
            }

            index += 1;
        }
    }

    ///
    /// Builds a pattern from a UTF-16 slice
    ///
    pub fn pattern_for_chars(regex: &[char]) -> Pattern<char> {
        // Characters to match exactly as built up so far
        let mut pattern         = vec![];
        let mut or_positions    = vec![];

        // Go through the slice and build up a regex
        let mut pos     = 0;
        let regex_len   = regex.len();

        while pos < regex_len {
            match regex[pos] {
                '\\' => {
                    // Quoted character
                    pos += 1;
                    if pos < regex_len {
                        pattern.push(LexTool::special_character_pattern(regex[pos]))
                    }
                },

                '.' => {
                    // Anything
                    pattern.push(MatchRange('\u{0000}', '\u{10ffff}'))
                },

                '*' => {
                    // Last item repeated
                    let pattern_len = pattern.len();
                    if pattern_len > 0 {
                        if let Some(last) = pattern.last().map(|x| x.clone()) {
                            pattern[pattern_len-1] = RepeatInfinite(0, Box::new(last));
                        }
                    }
                },

                '+' => {
                    // Last item at least once and then repeated
                    let pattern_len = pattern.len();
                    if pattern_len > 0 {
                        if let Some(last) = pattern.last().map(|x| x.clone()) {
                            pattern[pattern_len-1] = RepeatInfinite(1, Box::new(last));
                        }
                    }
                },

                '[' => {
                    // Character ranges
                    let mut ranges = vec![];
                    pos += 1;

                    let mut last_char = None;
                    while pos < regex_len && regex[pos] != ']' {
                        let mut next_char = regex[pos];

                        if next_char == '\\' && pos+1 < regex_len {
                            pos += 1;
                            next_char = LexTool::special_character_char(regex[pos]);
                        }

                        if next_char == '-' && pos < regex_len-1 {
                            pos += 1;
                            let final_char = regex[pos];

                            if let Some(last_char) = last_char {
                                ranges.last_mut().map(|x| *x = (last_char, final_char));
                            }
                        } else {
                            last_char = Some(next_char);
                            ranges.push((next_char, next_char));
                        }

                        pos += 1;
                    }

                    if ranges.len() == 1 {
                        let (first, last) = ranges[0];
                        pattern.push(MatchRange(first, last));
                    } else {
                        pattern.push(MatchAny(ranges.iter().map(|&(first, last)| MatchRange(first, last)).collect()));
                    }
                },

                '|' => {
                    // We'll join the two sides of the 'or' later on
                    or_positions.push(pattern.len());
                },

                '(' => {
                    // Subpattern
                    let subpattern = LexTool::get_subpattern(regex, pos);
                    pattern.push(LexTool::pattern_for_chars(subpattern));

                    pos += subpattern.len()+1;
                },

                c => {
                    // Just match this character
                    pattern.push(Match(vec![c]));
                }
            }

            // Next character
            pos += 1;
        }

        // Join up any subpatterns affected by the 'or' operator
        let mut offset = 0;
        for position_of_or in or_positions {
            if position_of_or > 0 {
                let actual_pos      = position_of_or-offset;
                let (left, right)   = (pattern[actual_pos-1].clone(), pattern[actual_pos].clone());

                pattern.remove(actual_pos);
                pattern[actual_pos-1] = MatchAny(vec![left, right]);

                offset += 1;
            }
        }

        // Join up plain matches
        LexTool::join_matches(&mut pattern);

        // Pattern that we've matched
        if pattern.len() == 0 {
            Epsilon
        } else if pattern.len() == 1 {
            pattern[0].clone()
        } else {
            MatchAll(pattern)
        }
    }
}

impl Tool for LexTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        // Attempt to parse the input
        let lex_defn = from_value::<LexToolInput>(input);

        // Fetch the tool for defining new tools in this environment
        let define_tool: Result<TypedTool<DefineToolInput, ()>, RetrieveToolError> = environment.get_json_tool(DEFINE_TOOL).map(|tool| TypedTool::from(tool));

        match (lex_defn, define_tool) {
            (Err(erm), _) => {
                // Fail if the input value doesn't deserialize
                Err(json![{
                    "error":        "Parameters incorrect",
                    "description":  erm.description()
                }])
            },

            (_, Err(erm)) => {
                // Fail if there's no define tool
                Err(json![{
                    "error":        "Could not retrieve define-tool",
                    "description":  erm.message()
                }])
            },

            (Ok(lex_defn), Ok(define_tool)) => {
                // Generate a lexer tool for this definition
                let lexer_tool = StringLexingTool::from_lex_tool_input(&lex_defn);

                // Create an environment with just the tool
                let lexer_toolset   = BasicToolSet::from(vec![ ("lex", lexer_tool) ]);
                let lexer_env       = StaticEnvironment::from_toolset(lexer_toolset, &EmptyEnvironment::new());

                // Define it in the environment
                define_tool.invoke(DefineToolInput::new("new", Some(&lex_defn.new_tool_name)), &lexer_env).map(|_| Value::Null)
            }
        }
    }
}

///
/// Tool that reads a string and generates a lexed array of matches
///
pub struct StringLexingTool {
    matcher: TokenMatcher<char, String>
}

impl StringLexingTool {
    ///
    /// Creates a lexer tool from a definition
    ///
    pub fn from_lex_tool_input(lex_defn: &LexToolInput) -> StringLexingTool {
        // Generate a token matcher from the lexer
        let mut token_matcher = TokenMatcher::new();

        for symbol in lex_defn.symbols.iter() {
            token_matcher.add_pattern(LexTool::pattern_for_string(&symbol.match_rule), symbol.symbol_name.clone());
        }

        token_matcher.prepare_to_match();

        // This is what we use in the lexing tool
        StringLexingTool { matcher: token_matcher }
    }
}

impl Tool for StringLexingTool {
    fn invoke_json(&self, input: Value, environment: &Environment) -> Result<Value, Value> {
        if let Value::String(input) = input {
            // Input must be a simple string

            // Start tokenizing it
            let mut tokenizer   = Tokenizer::new(input.read_symbols(), &self.matcher);
            let mut result      = vec![];

            while let Some((range, token)) = tokenizer.next() {
                result.push(json![ { 
                    "token":    token,
                    "matched":  input[range.clone()],
                    "start":    range.start,
                    "end":      range.end
                } ]);
            }

            // Final result is an array of tokens
            Ok(Value::Array(result))
        } else {
            Err(json![{
                "error": "Input must be a string"
            }])
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_phrase_match() {
        assert!(LexTool::pattern_for_string("phrase") == Match(vec!['p', 'h', 'r', 'a', 's', 'e']));
    }

    #[test]
    fn can_create_any_pattern() {
        assert!(LexTool::pattern_for_string(".*") == RepeatInfinite(0, Box::new(MatchRange('\u{0000}', '\u{10ffff}'))));
    }

    #[test]
    fn can_create_or_match() {
        assert!(LexTool::pattern_for_string("a|b") == MatchAny(vec![ Match(vec!['a']), Match(vec!['b']) ]));
    }

    #[test]
    fn can_create_nested_or_match() {
        assert!(LexTool::pattern_for_string("a|b|c") == MatchAny(vec![ MatchAny(vec![ Match(vec!['a']), Match(vec!['b']) ]), Match(vec!['c'])] ));
    }

    #[test]
    fn can_create_grouped_or_match() {
        assert!(LexTool::pattern_for_string("(foo)|(bar)") == MatchAny(vec![ Match(vec!['f', 'o', 'o']), Match(vec!['b', 'a', 'r']) ]));
    }

    #[test]
    fn or_is_processed_early() {
        assert!(LexTool::pattern_for_string("foo|bar") == MatchAll(vec![ Match(vec!['f', 'o']), MatchAny(vec![ Match(vec!['o']), Match(vec!['b']) ]), Match(vec!['a', 'r']) ]));
    }

    #[test]
    fn can_create_simple_grouping() {
        assert!(LexTool::pattern_for_string("(phrase)") == Match(vec!['p', 'h', 'r', 'a', 's', 'e']));
    }

    #[test]
    fn can_create_nested_grouping() {
        assert!(LexTool::pattern_for_string("(p(h(r)a)s)e") == Match(vec!['p', 'h', 'r', 'a', 's', 'e']));
    }

    #[test]
    fn can_create_match_one() {
        assert!(LexTool::pattern_for_string("[a]") == MatchRange('a', 'a'));
    }

    #[test]
    fn can_interpret_newline_quote_characters() {
        assert!(LexTool::pattern_for_string("\\n") == Match(vec![ '\n' ]));
    }

    #[test]
    fn can_create_match_range() {
        assert!(LexTool::pattern_for_string("[a-z]") == MatchRange('a', 'z'));
    }

    #[test]
    fn can_create_match_set() {
        assert!(LexTool::pattern_for_string("[acgh]") == MatchAny(vec![ MatchRange('a', 'a'), MatchRange('c', 'c'), MatchRange('g', 'g'), MatchRange('h', 'h') ]));
    }

    #[test]
    fn can_create_match_multi_range() {
        assert!(LexTool::pattern_for_string("[a-zA-Z]") == MatchAny(vec![ MatchRange('a', 'z'), MatchRange('A', 'Z') ]));
    }

    #[test]
    fn can_create_match_set_and_range() {
        assert!(LexTool::pattern_for_string("[aA-Z]") == MatchAny(vec![ MatchRange('a', 'a'), MatchRange('A', 'Z') ]));
    }
}

