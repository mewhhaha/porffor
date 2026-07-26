use crate::{
    lexer::{Token, TokenKind},
    parser::cursor::BufferedLexer,
};
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn peek_skip_accending() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
}

#[test]
fn peek_has_no_fixed_token_limit() {
    let mut cur = BufferedLexer::from(
        &b"a0 a1 a2 a3 a4 a5 a6 a7 a8 a9 a10 a11 a12 a13 a14 a15 a16 a17 a18 a19 a20 a21 a22 a23 a24 a25 a26 a27 a28 a29 a30 a31 a32 a33 a34 a35 a36 a37 a38 a39"[..],
    );
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(39, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a39", utf16!("a39")))
    );
}

#[test]
fn peek_skip_next() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d", utf16!("d")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("e", utf16!("e")))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f", utf16!("f")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("g", utf16!("g")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("h", utf16!("h")))
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d", utf16!("d")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f", utf16!("f")))
    );
}

#[test]
fn peek_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    loop {
        let peek = cur.peek(0, false, interner).unwrap().cloned();
        let next = cur.next(false, interner).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    let mut peeked = [None::<Token>, None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false, interner).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false, interner).unwrap(), peek);
        }

        if peeked[2].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut cur = BufferedLexer::from(&b"A \n B"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A", utf16!("A")))
    );
    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A", utf16!("A")))
    );

    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B", utf16!("B"))) /* This value is after the line terminator */
    );

    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B", utf16!("B")))
    );
    // End of stream
    assert!(cur.peek(2, true, interner).unwrap().is_none());
}

#[test]
fn issue_1768() {
    let mut cur = BufferedLexer::from(&b"\n(\nx\n)\n"[..]);
    let interner = &mut Interner::default();

    assert!(cur.peek(3, true, interner).unwrap().is_none());
}
