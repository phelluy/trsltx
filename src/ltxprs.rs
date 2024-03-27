//! This library is a parser for a subset of LateX.
//! It uses the nom library.
//! It can extract a W3C EBNF grammar of the latex parsed chunk.

//the grammar of the chunk is as follows
// the list of possible commands is added afterward
#[allow(dead_code)]
const GRAMAR: &str = r#"
# W3C EBNF grammar of the Latex chunk
root ::= "\\begin{trsltx}" stuff "\\end{trsltx}"
stuff ::= (atom | construct)*
atom ::= command | text
construct ::= group | math
text ::= [^\\{}$%]+
group ::= "{" stuff "}"
math ::= ("$" stuff "$") | ("$$" stuff "$$")"#;
// example of a missing command line that will be added by the grammar
//command ::= "\\item"  | "\\begin" | "\\frac" | "\\label{eq:formule}" |
// "\\end" | "\\ref//{eq:autre_formule}" | "\\section" | "\\sqrt"

// import exit function for debugging (sometimes)
#[allow(unused_imports)]
use std::process::exit;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, char, none_of},
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::{delimited, preceded},
};

///The recursive structure that contains the whole AST
/// Remark: the Text node may contain \begin{} ... \end{} environments
/// including maths. Only the $...$ and $$...$$ are checked for now.
#[derive(Debug, PartialEq, Clone)]
pub enum LtxNode {
    Text(String),              // a text without any special character (no \{}$%)
    Comment(String),           // a comment starting with a % and ending with a \n
    Label(String),             // a label starting with \label{ and ending with }
    Reference(String),         // a reference starting with \ref{ and ending with }
    Command(String),           // a command starting with a \ and followed by [a-zA-Z]+ or [\&{}[]]
    Group(Vec<LtxNode>),       // a group of nodes between { and }
    Math(Vec<LtxNode>),        // a math environment between $ and $ or \( and \)
    DisplayMath(Vec<LtxNode>), // a display math environment between $$ and $$ or \[ and \]
}

impl LtxNode {
    pub fn new(s: &str) -> LtxNode {
        let s = s.trim();
        // construct the string {s} so that the head Node is a group.
        // the \n's are important for parsing initial or closing %'s
        let s = format!("{{\n{}\n}}", s);
        //println!("new: {}", s);
        group_node(&s).unwrap().1
    }

    ///Iters in the ltxnode and extracts all the command names
    pub fn extracts_commands(&self) -> Vec<String> {
        let mut cmd_list = vec![];
        match self {
            LtxNode::Text(_) => (),
            LtxNode::Comment(_) => (),
            LtxNode::Label(_) => (),
            LtxNode::Reference(_) => (),
            LtxNode::Command(s) => cmd_list.push(s.clone()),
            LtxNode::Group(v) => {
                for n in v {
                    cmd_list.append(&mut n.extracts_commands());
                }
            }
            LtxNode::Math(v) => {
                for n in v {
                    cmd_list.append(&mut n.extracts_commands());
                }
            }
            LtxNode::DisplayMath(v) => {
                for n in v {
                    cmd_list.append(&mut n.extracts_commands());
                }
            }
        }
        // remove repeated entries
        cmd_list.sort();
        cmd_list.dedup();
        cmd_list
    }

    ///Iters in the ltxnode and extracts all the labels
    pub fn extracts_labels(&self) -> Vec<String> {
        let mut label_list = vec![];
        match self {
            LtxNode::Text(_) => (),
            LtxNode::Comment(_) => (),
            LtxNode::Command(_) => (),
            LtxNode::Reference(_) => (),
            LtxNode::Label(s) => label_list.push(s.clone()),
            LtxNode::Group(v) => {
                for n in v {
                    label_list.append(&mut n.extracts_labels());
                }
            }
            LtxNode::Math(v) => {
                for n in v {
                    label_list.append(&mut n.extracts_labels());
                }
            }
            LtxNode::DisplayMath(v) => {
                for n in v {
                    label_list.append(&mut n.extracts_labels());
                }
            }
        }
        // remove repeated entries
        label_list.sort();
        label_list.dedup();
        label_list
    }

    ///Iters in the ltxnode and extracts all the references
    pub fn extracts_references(&self) -> Vec<String> {
        let mut ref_list = vec![];
        match self {
            LtxNode::Text(_) => (),
            LtxNode::Comment(_) => (),
            LtxNode::Command(_) => (),
            LtxNode::Label(_) => (),
            LtxNode::Reference(s) => ref_list.push(s.clone()),
            LtxNode::Group(v) => {
                for n in v {
                    ref_list.append(&mut n.extracts_references());
                }
            }
            LtxNode::Math(v) => {
                for n in v {
                    ref_list.append(&mut n.extracts_references());
                }
            }
            LtxNode::DisplayMath(v) => {
                for n in v {
                    ref_list.append(&mut n.extracts_references());
                }
            }
        }
        // remove repeated entries
        ref_list.sort();
        ref_list.dedup();
        ref_list
    }

    /// Generate the W3C EBNF grammar of the latex chunk
    pub fn to_ebnf(&self) -> String {
        let s0 = GRAMAR.to_string();
        // append to s all the labels and command separated by "|"
        // on a single line, do it so that the backslashes are not removed
        let labels = self.extracts_labels();
        let refs = self.extracts_references();
        let cmds = self.extracts_commands();
        // add a fake command for avoiding an empty list of commands
        let mut s = "\ncommand ::= \"\\commandevide\" | ".to_string();
        for l in labels {
            s = s + "\"" + l.clone().as_str() + "\"" + " | ";
        }
        for r in refs {
            s = s + "\"" + r.clone().as_str() + "\"" + " | ";
        }
        for c in cmds {
            s = s + "\"" + c.clone().as_str() + "\"" + " | ";
        }
        // remove the trailing " | "
        s = s.trim_end_matches(" | ").to_string();
        // replace all the "\" by "\\"
        s = s.replace("\\", "\\\\");
        let s = s0 + &s;
        //println!("{}", s);
        s
    }
}

///parse a text until one of these character is encountered: \{}$%
fn text(input: &str) -> nom::IResult<&str, &str> {
    recognize(many1(none_of("\\{}$%")))(input)
}

///parse a text and produce a LtxNode::Text
fn text_node(input: &str) -> nom::IResult<&str, LtxNode> {
    map(text, |s: &str| LtxNode::Text(s.to_string()))(input)
}

// ///parse a string that is neither  "ref" nor "label"
// fn no_ref_label_str(input: &str) -> nom::IResult<&str, &str> {
//     permutation((recognize(many1(none_of("label"))), recognize(many1(none_of("ref")))))(input)
// }

// // parse a string that is not "ref" and not "label" using the previous parser
// fn not_ref_label_str(input: &str) -> nom::IResult<&str, &str> {

// }

// parse an ascii command: a backslash followed by a string of letters
fn ascii_cmd(input: &str) -> nom::IResult<&str, &str> {
    preceded(char('\\'), alpha1)(input)
}

//parse an alphatext with this possible character: :-_
// fn label_text(input: &str) -> nom::IResult<&str, &str> {
//     recognize(many1(alt((alpha1, tag(":"), tag("-"), tag("_")))))(input)
// }
// label_text parser same as text parser
fn label_text(input: &str) -> nom::IResult<&str, &str> {
    recognize(many1(none_of("\\{}$%")))(input)
}

///parse a label_text enclosed in braces
fn label_braces(input: &str) -> nom::IResult<&str, &str> {
    delimited(char('{'), label_text, char('}'))(input)
}

///parse a label: a label_text with a \label prefix
fn label(input: &str) -> nom::IResult<&str, &str> {
    preceded(tag("\\label"), label_braces)(input)
}

///parse a ref: a label_text with a \ref prefix
fn ltxref(input: &str) -> nom::IResult<&str, &str> {
    preceded(tag("\\ref"), label_braces)(input)
}

///LtxNode version of the previous function
fn ltxref_node(input: &str) -> nom::IResult<&str, LtxNode> {
    map(ltxref, |s: &str| {
        // prepend \ref{ and append }
        let cs = format!("\\ref{{{}}}", s);
        LtxNode::Reference(cs.to_string())
    })(input)
}

///LtxNode version of the label parser
fn label_node(input: &str) -> nom::IResult<&str, LtxNode> {
    map(label, |s: &str| {
        // prepend \label{ and append }
        let cs = format!("\\label{{{}}}", s);
        LtxNode::Label(cs.to_string())
    })(input)
}

///Parse a backslash followed by a special character: \{}()[]$&,;%@:-
/// and the accented characters: '`^"~
fn backslash_special(input: &str) -> nom::IResult<&str, &str> {
    alt((
        tag("\\\\"),
        tag("\\{"),
        tag("\\}"),
        tag("\\("),
        tag("\\)"),
        tag("\\["),
        tag("\\]"),
        tag("\\$"),
        tag("\\&"),
        tag("\\,"),
        tag("\\;"),
        tag("\\%"),
        tag("\\@"),
        tag("\\:"),
        tag("\\-"),
        tag("\\'"),
        tag("\\`"),
        tag("\\^"),
        tag("\\\""),
        tag("\\~"),
    ))(input)
    //tag("\\\\")(input)
}
// fn backslash_special(input: &str) -> nom::IResult<&str, &str> {
//     alt((tag("\\\\"), tag("\\$"), tag("\\&")))(input)
// }

///parse an ascii_cmd or a backslash_special
fn command(input: &str) -> nom::IResult<&str, &str> {
    alt((ascii_cmd, backslash_special))(input)
}

///parse a command and produce a LtxNode::Command
fn command_node(input: &str) -> nom::IResult<&str, LtxNode> {
    map(command, |s: &str| {
        // add "\\" at the beginning of the command
        // if the string is not already a backslash_special
        let cs = if s.starts_with("\\") {
            s.to_string()
        } else {
            format!("\\{}", s)
        };
        LtxNode::Command(cs.to_string())
    })(input)
}

///parse until end of line
fn end_of_line(input: &str) -> nom::IResult<&str, &str> {
    recognize(many0(none_of("\n")))(input)
}

///parse a comment: anything between a % and a \n
fn comment(input: &str) -> nom::IResult<&str, &str> {
    preceded(tag("%"), end_of_line)(input)
}

///parse a comment and produce a LtxNode::Comment
fn comment_node(input: &str) -> nom::IResult<&str, LtxNode> {
    map(comment, |s: &str| {
        //println!("comment");
        LtxNode::Comment(s.to_string())
    })(input)
}

///parse a math node delimited by $ .. $ or \( .. \)
fn math_node(input: &str) -> nom::IResult<&str, LtxNode> {
    //println!("math");
    alt((
        map(
            delimited(tag("$"), many0(alt((atom_node, group_node))), tag("$")),
            |v| LtxNode::Math(v),
        ),
        map(
            delimited(tag("\\("), many0(alt((atom_node, group_node))), tag("\\)")),
            |v| LtxNode::Math(v),
        ),
    ))(input)
}

///parse a display math node delimited by $$ .. $$ or \[ .. \]
fn display_math_node(input: &str) -> nom::IResult<&str, LtxNode> {
    //println!("display math");
    alt((
        map(
            delimited(tag("$$"), many0(alt((atom_node, group_node))), tag("$$")),
            |v| LtxNode::DisplayMath(v),
        ),
        map(
            delimited(tag("\\["), many0(alt((atom_node, group_node))), tag("\\]")),
            |v| LtxNode::DisplayMath(v),
        ),
    ))(input)
}

///parse an atom, which is a command, a comment or a text or a math env
/// some remarks: math envs cannot be nested
fn atom_node(input: &str) -> nom::IResult<&str, LtxNode> {
    alt((
        comment_node, // the order is important
        text_node,
        ltxref_node,
        label_node,
        command_node,
    ))(input)
}

///parse a group of nodes recursively
fn group_node(input: &str) -> nom::IResult<&str, LtxNode> {
    //println!("recursing");
    map(
        delimited(
            char('{'),
            many0(alt((atom_node, group_node, math_node, display_math_node))),
            char('}'),
        ),
        |v| LtxNode::Group(v),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    // cargo command for running a test <nametest>
    // with results displayed
    // cargo test <nametest> -- --nocapture
    #[test]
    fn parse_text() {
        let str = "oulaOula";
        let res = text(str);
        assert_eq!(res, Ok(("", "oulaOula")));
        let str = "oulaOula%";
        let res = text_node(str);
        assert_eq!(res, Ok(("%", LtxNode::Text("oulaOula".to_string()))));
        assert_eq!(text("oulaOula%"), Ok(("%", "oulaOula")));
        assert_eq!(text("oula\\Oula"), Ok(("\\Oula", "oula")));
    }

    #[test]
    fn parse_ascii_cmd() {
        let str = "\\oula";
        let res = ascii_cmd(str);
        println!("{:?}", res);
        assert_eq!(res, Ok(("", "oula")));
        assert_eq!(ascii_cmd("\\oulaé%"), Ok(("é%", "oula")));
    }

    #[test]
    fn parse_end_of_line() {
        let str = "oulaOula\n";
        let res = end_of_line(str);
        assert_eq!(res, Ok(("\n", "oulaOula")));
        assert_eq!(end_of_line("oulaOula%"), Ok(("", "oulaOula%")));
        assert_eq!(end_of_line("oulaOula\\Oula"), Ok(("", "oulaOula\\Oula")));
    }

    #[test]
    fn parse_command() {
        let str = "\\oula";
        let res = command(str);
        println!("{:?}", res);
        assert_eq!(res, Ok(("", "oula")));
        assert_eq!(command("\\oulaé%"), Ok(("é%", "oula")));
        assert_eq!(command("\\\\oulaé%"), Ok(("oulaé%", "\\\\")));
    }

    #[test]
    fn parse_comment() {
        // let str = "aaaa%oula\n";
        // // assert comment(str) generates an error
        //assert_eq!(comment(str), Ok(("", str)));
        let str = "%oula\n";
        assert_eq!(comment(str), Ok(("\n", "oula")));
    }

    #[test]
    fn parse_atom() {
        let str = "aaaa%oula\n";
        assert_eq!(
            atom_node(str),
            Ok(("%oula\n", LtxNode::Text("aaaa".to_string())))
        );
        let str = "%oula\n\\toto";
        assert_eq!(
            atom_node(str),
            Ok(("\n\\toto", LtxNode::Comment("oula".to_string())))
        );
        let str = "\\oulaé";
        assert_eq!(
            atom_node(str),
            Ok(("é", LtxNode::Command("\\oula".to_string())))
        );
    }

    #[test]
    fn parse_group_node() {
        let str = "{\\item salut ça va ? % ouf tout va bien\n}";
        let grp = group_node(str);
        println!("{:?}", grp);
        assert_eq!(
            grp,
            Ok((
                "",
                LtxNode::Group(vec![
                    LtxNode::Command("\\item".to_string()),
                    LtxNode::Text(" salut ça va ? ".to_string()),
                    LtxNode::Comment(" ouf tout va bien".to_string()),
                    LtxNode::Text("\n".to_string()),
                ])
            ))
        );
    }

    #[test]
    fn recursive_test() {
        let str = r#"{
\item a
% rien 
\item {\blue b}
}
        "#;
        let grp = group_node(str);
        println!("{:?}", grp);
    }

    #[test]
    fn new_ltx_test() {
        let str = r#"
\ref{toto}        
\item a \\
% rien 
\label{toto}
\item {\blue {\b \ref{tata} \label{titi}}}
              
              "#;
        let latex = LtxNode::new(str);
        println!("{:?}", latex);
        let cmds = latex.extracts_commands();
        println!("commands: {:?}", cmds);
        let labels = latex.extracts_labels();
        println!("labels: {:?}", labels);
        let refs = latex.extracts_references();
        println!("references: {:?}", refs);
    }

    #[test]
    fn test_math() {
        let str = r#"
\toto
\[ \int_0^1 f(x) dx \]
$ \frac{1}{2}$
"#;
        let latex = LtxNode::new(str);
        println!("{:?}", latex);
        let cmds = latex.extracts_commands();
        println!("commands: {:?}", cmds);
        let labels = latex.extracts_labels();
        println!("labels: {:?}", labels);
        let refs = latex.extracts_references();
        println!("references: {:?}", refs);
    }

    #[test]
    fn test_full() {
        let str = r#"
% comment
\ref{toto}        
\item a \\
$ \frac{a}{b} $
\label{toto}
\item {\blue {\b \ref{tata} \label{titi}}}
 "#;
        let latex = LtxNode::new(str);
        println!("{:?}", latex);
        let cmds = latex.extracts_commands();
        println!("commands: {:?}", cmds);
        let labels = latex.extracts_labels();
        println!("labels: {:?}", labels);
        let refs = latex.extracts_references();
        println!("references: {:?}", refs);
        println!("{}", latex.to_ebnf());
    }
}
