//! Whole-request cache exclusion for native external-input macros. This is
//! an input proof only; the original compiler still checks the complete source.
use crate::{diag::FileId,intern::Interner,lexer::{self,Tok}};

pub(super) fn reads_outside(source:&[u8])->bool {
    // The common path needs no second lex. These are conservative byte
    // candidates, including names separated from `!` by comments or supplied
    // as macro arguments. Strings and comments alone need not disable reuse.
    if ![b"env".as_slice(),b"include".as_slice()].iter().any(|word|source.windows(word.len()).any(|part|part==*word)) {
        return false;
    }
    let Ok(source)=std::str::from_utf8(source) else{return true};
    let interner=Interner::default();
    let Ok(lexed)=lexer::lex(source,FileId(0),0,&interner) else{return true};
    lexed.tokens.iter().any(|token|match token.tok {
        Tok::Ident(name)=>matches!(interner.get(name),"env"|"option_env"|"include"|"include_str"|"include_bytes"),
        _=>false,
    })
}
