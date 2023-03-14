use crate::clvm::assemble::assemble_text;
use crate::clvm::assemble::keywords::KEYWORD_TO_ATOM;
use crate::clvm::parser::{sexp_from_bytes, sexp_to_bytes};
use crate::clvm::program::SerializedProgram;
use crate::clvm::program::{Program, NULL};
use crate::clvm::sexp::AtomBuf;
use crate::clvm::sexp::SExp;
use hex::encode;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Error;
use std::io::ErrorKind;

lazy_static! {
    pub static ref CURRY_OBJ_CODE: SerializedProgram = assemble_text("(a (q #a 4 (c 2 (c 5 (c 7 0)))) (c (q (c (q . 2) (c (c (q . 1) 5) (c (a 6 (c 2 (c 11 (q 1)))) 0))) #a (i 5 (q 4 (q . 4) (c (c (q . 1) 9) (c (a 6 (c 2 (c 13 (c 11 0)))) 0))) (q . 11)) 1) 1))").expect("Static Assemble Should not fail.");
    pub static ref UNCURRY_PATTERN_FUNCTION: SerializedProgram = assemble_text("(a (q . (: . function)) (: . core))").expect("Static Assemble Should not fail.");
    pub static ref UNCURRY_PATTERN_CORE: SerializedProgram = assemble_text("(c (q . (: . parm)) (: . core))").expect("Static Assemble Should not fail.");
}

#[tokio::test]
async fn test_curry_assemble() {
    let cur_prog = CURRY_OBJ_CODE.clone();
    println!("Curry prog: {}", encode(&cur_prog.to_bytes()));
}

const BYTE_MATCH: [u8; 1] = [81u8];
const ATOM_MATCH: [u8; 1] = [b'$'];
const SEXP_MATCH: [u8; 1] = [b':'];

pub fn uncurry(
    curried_program: &SerializedProgram,
) -> Result<Option<(SerializedProgram, SerializedProgram)>, Error> {
    let pattern_func = sexp_from_bytes(UNCURRY_PATTERN_FUNCTION.to_bytes())?;
    let pattern_core = sexp_from_bytes(UNCURRY_PATTERN_CORE.to_bytes())?;
    let sexp = sexp_from_bytes(curried_program.to_bytes())?;
    match match_sexp(&pattern_func, &sexp, HashMap::new()) {
        Some(mut func_results) => {
            let func = func_results.remove("function").ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    "Failed to find function in curried program",
                )
            })?;
            let mut core = func_results.remove("core").ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    "Failed to find core in curried program",
                )
            })?;
            let mut args: Vec<SExp> = Vec::new();
            while let Some(mut core_results) = match_sexp(&pattern_core, &core, HashMap::new()) {
                args.push(core_results.remove("parm").ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        "Failed to find parm in curried program",
                    )
                })?);
                core = core_results.remove("core").ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidInput,
                        "Failed to find core in curried program",
                    )
                })?;
            }
            match core {
                SExp::Atom(buf) => {
                    if buf.data == BYTE_MATCH {
                        Ok(Some((
                            SerializedProgram::from_bytes(&sexp_to_bytes(&func)?),
                            SerializedProgram::from_bytes(&sexp_to_bytes(&concat(&args)?)?),
                        )))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        }
        None => Ok(None),
    }
}

pub fn concat(sexps: &[SExp]) -> Result<SExp, Error> {
    let mut buf = AtomBuf::new(vec![]);
    for sexp in sexps {
        match sexp {
            SExp::Atom(a) => {
                buf.data.extend(&a.data);
            }
            SExp::Pair(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "(internal error) concat expected atom, got pair",
                ));
            }
        }
    }
    Ok(SExp::Atom(buf))
}

pub fn curry(program: &Program, args: &[Program]) -> Result<Program, Error> {
    let args = make_args(args)?;
    println!("Args prog: {:?}", &args.serialized);
    println!("Args prog: {}", encode(&args.serialized));
    println!("Args prog: {}", args);
    let as_str = format!("(a (q . {program}) {args})");
    println!("as_str: {}", as_str);
    let serialized = assemble_text(&as_str)?;
    let program = serialized.to_program()?;
    println!("serialized: {}", serialized);
    println!("program: {}", program);
    Ok(program)
}

fn make_args(args: &[Program]) -> Result<Program, Error> {
    let mut cur = Program::new(KEYWORD_TO_ATOM.get("q").unwrap().clone());
    for arg in args.iter().rev() {
        cur = cons(
            &Program::new(KEYWORD_TO_ATOM.get("c").unwrap().clone()),
            &cons(
                &cons(
                    &Program::new(KEYWORD_TO_ATOM.get("q").unwrap().clone()),
                    arg,
                ),
                &cons(&cur, &NULL),
            ),
        );
    }
    Ok(cur)
}

fn cons(first: &Program, other: &Program) -> Program {
    first.cons(other)
}

//
// fn make_args(args: &Vec<Program>) -> Result<Program, Error> {
//     if args.is_empty() {
//         return Ok(Program::null());
//     }
//     let mut rtn = args
//         .last()
//         .ok_or_else(|| {
//             Error::new(
//                 ErrorKind::Other,
//                 "Not Last Element with Non Empty List (Should not happen)",
//             )
//         })?
//         .cons(&Program::null());
//     for arg in args[1..=args.len() - 1].iter().rev() {
//         rtn = arg.cons(&rtn);
//     }
//     Ok(rtn)
// }

pub fn match_sexp<'a>(
    pattern: &'a SExp,
    sexp: &'a SExp,
    known_bindings: HashMap<String, SExp>,
) -> Option<HashMap<String, SExp>> {
    match (pattern, sexp) {
        (SExp::Atom(pat_buf), SExp::Atom(sexp_buf)) => {
            if pat_buf == sexp_buf {
                Some(known_bindings)
            } else {
                None
            }
        }
        (SExp::Pair(pair), _) => match (pair.first.as_ref(), pair.rest.as_ref()) {
            (SExp::Atom(pat_left), SExp::Atom(pat_right)) => match sexp {
                SExp::Atom(sexp_buf) => {
                    if pat_left.data == ATOM_MATCH.to_vec() {
                        if pat_right.data == ATOM_MATCH.to_vec() {
                            if sexp_buf.data == ATOM_MATCH.to_vec() {
                                return Some(HashMap::new());
                            }
                            return None;
                        }

                        return unify_bindings(known_bindings, &pat_right.data, sexp);
                    }
                    if pat_left.data == SEXP_MATCH.to_vec() {
                        if pat_right.data == SEXP_MATCH.to_vec()
                            && sexp_buf.data == SEXP_MATCH.to_vec()
                        {
                            return Some(HashMap::new());
                        }

                        return unify_bindings(known_bindings, &pat_right.data, sexp);
                    }

                    None
                }
                SExp::Pair(spair) => {
                    if pat_left.data == SEXP_MATCH.to_vec() && pat_right.data != SEXP_MATCH.to_vec()
                    {
                        return unify_bindings(known_bindings, &pat_right.data, sexp);
                    }
                    match_sexp(&pair.first, &spair.first, known_bindings)
                        .and_then(|new_bindings| match_sexp(&pair.rest, &spair.rest, new_bindings))
                }
            },
            _ => match sexp {
                SExp::Atom(_) => None,
                SExp::Pair(spair) => match_sexp(&pair.first, &spair.first, known_bindings)
                    .and_then(|new_bindings| match_sexp(&pair.rest, &spair.rest, new_bindings)),
            },
        },
        (SExp::Atom(_), _) => None,
    }
}

pub fn unify_bindings<'a>(
    mut bindings: HashMap<String, SExp>,
    new_key: &'a [u8],
    new_value: &'a SExp,
) -> Option<HashMap<String, SExp>> {
    let new_key_str = String::from_utf8_lossy(new_key).as_ref().to_string();
    match bindings.get(&new_key_str) {
        Some(binding) => {
            if !equal_to(binding, new_value) {
                return None;
            }
            Some(bindings)
        }
        _ => {
            bindings.insert(new_key_str, new_value.clone());
            Some(bindings)
        }
    }
}

pub fn equal_to(first: &SExp, second: &SExp) -> bool {
    match (first, second) {
        (SExp::Atom(fbuf), SExp::Atom(sbuf)) => fbuf == sbuf,
        (SExp::Pair(first), SExp::Pair(rest)) => {
            equal_to(&first.first, &rest.first) && equal_to(&first.rest, &rest.rest)
        }
        _ => false,
    }
}
