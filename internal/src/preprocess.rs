use proc_macro2::{Delimiter, Group, Punct, TokenStream, TokenTree};
use std::mem;
use syn::__private::quote::quote;
use syn::__private::TokenStreamExt;

pub fn preprocess(input: TokenStream) -> TokenStream {
    let mut output = TokenStream::new();
    let mut state = State::Ready;
    for tree in input {
        let tree = match mem::take(&mut state) {
            State::Ready => {
                if let TokenTree::Ident(ident) = tree {
                    state = State::Base(ident.into());
                    continue;
                } else if let TokenTree::Group(group) = &tree && group.delimiter() == Delimiter::Parenthesis {
                    let TokenTree::Group(group) = tree else {
                        unreachable!()
                    };
                    state = State::Base(group.into());
                    continue;
                } else {
                    tree
                }
            }
            State::Base(ident) => {
                if let TokenTree::Punct(punct) = &tree && punct.as_char() == '.' {
                    let TokenTree::Punct(punct) = tree else {
                        unreachable!()
                    };
                    state = State::Dot(ident, punct);
                    continue;
                } else {
                    output.append(ident);
                    tree
                }
            }
            State::Dot(ident, punct) => {
                if let TokenTree::Group(group) = &tree && group.delimiter() == Delimiter::Bracket
                {
                    let TokenTree::Group(group) = tree else {
                        unreachable!()
                    };
                    output.extend(quote!(__tuple_index!((#ident), #group)));
                    continue;
                } else {
                    output.append(ident);
                    output.append(punct);
                    tree
                }
            }
        };
        if let TokenTree::Group(group) = tree {
            output.append(Group::new(group.delimiter(), preprocess(group.stream())));
        } else {
            output.append(tree);
        }
    }
    match state {
        State::Ready => {}
        State::Base(ident) => {
            output.append(ident);
        }
        State::Dot(ident, punct) => {
            output.append(ident);
            output.append(punct);

        }
    }
    output
}

#[derive(Default)]
enum State {
    #[default]
    Ready,
    Base(TokenTree),
    Dot(TokenTree, Punct),
}
