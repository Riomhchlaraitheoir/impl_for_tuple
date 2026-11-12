mod preprocess;
#[cfg(test)]
mod test;

use crate::preprocess::preprocess;
use proc_macro2::{Ident, Span, TokenStream};
use std::collections::BTreeMap;
use std::mem;
use syn::__private::ToTokens;
use syn::__private::quote::format_ident;
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{Attribute, BinOp, Block, Expr, ExprAssign, ExprBlock, ExprField, ExprForLoop, ExprLit, ExprTuple, GenericArgument, GenericParam, ItemImpl, Lit, LitInt, Local, LocalInit, Member, Meta, PathArguments, PathSegment, PredicateType, RangeLimits, Stmt, Token, TraitBound, Type, TypeParam, TypeParamBound, TypePath, TypeTuple, WhereClause, WherePredicate, bracketed, parenthesized, parse2, visit_mut};

pub fn impl_for_tuple(input: TokenStream) -> syn::Result<TokenStream> {
    let input = preprocess(input);

    println!("{}", input);

    let mut input: ItemImpl = parse2(input).with_message("failed to parse impl block:")?;

    let mut min_count = 0;
    let mut max_count = 12; // using 12 as the default upper limit since that is how the stdlib traits are done
    if let Type::Path(ty) = &*input.self_ty
        && ty.path.segments.len() == 1
        && ty.path.segments[0].ident == "Tuple"
    {
        if let PathArguments::AngleBracketed(args) = &ty.path.segments[0].arguments {
            if args.args.len() > 1 {
                return Err(syn::Error::new(
                    args.span(),
                    "only a single arg is supported here",
                ));
            }
            if let Some(range) = args.args.get(0) {
                let span = range.span();
                if let GenericArgument::Const(range) = range {
                    let IndexRange {
                        span: _,
                        start,
                        end,
                    } = eval_range(range, None)?;
                    if let Some(start) = start {
                        min_count = start;
                    }
                    if let Some(end) = end {
                        max_count = end - 1;
                    } else {
                        return Err(syn::Error::new(
                            range.span(),
                            "cannot use open ended range here",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        span,
                        "range arg must be wrapper in a block",
                    ));
                }
            }
        }
    } else {
        return Err(syn::Error::new(
            input.self_ty.span(),
            "Self type must be 'Tuple'",
        ));
    }
    let additional_tuples = {
        let mut additional_tuples = vec![];
        let where_clause = input
            .generics
            .where_clause
            .get_or_insert_with(|| WhereClause {
                where_token: Default::default(),
                predicates: Default::default(),
            });
        input.generics.params = mem::take(&mut input.generics.params)
            .into_iter()
            .map(|param| -> syn::Result<Option<GenericParam>> {
                let GenericParam::Type(param) = param else {
                    return Ok(Some(param));
                };
                let parsed_bounds = param
                    .bounds
                    .iter()
                    .map(as_tuple_bound)
                    .collect::<syn::Result<Vec<_>>>()?;
                if parsed_bounds.iter().any(|tuple_bound| {
                    if let Some(length) = tuple_bound {
                        additional_tuples.push((param.ident.clone(), length.clone()));
                        true
                    } else {
                        false
                    }
                }) {
                    let mut bounds = param.bounds;
                    bounds = bounds
                        .into_iter()
                        .zip(parsed_bounds)
                        .filter_map(|(bound, tuple_bound)| tuple_bound.is_none().then_some(bound))
                        .collect();
                    if !bounds.is_empty() {
                        where_clause
                            .predicates
                            .push(WherePredicate::Type(PredicateType {
                                attrs: vec![],
                                lifetimes: None,
                                bounded_ty: Type::Path(TypePath {
                                    attrs: vec![],
                                    qself: None,
                                    path: param.ident.into(),
                                }),
                                colon_token: Default::default(),
                                bounds,
                            }));
                    }
                    Ok(None)
                } else {
                    Ok(Some(GenericParam::Type(param)))
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        additional_tuples
    };
    (min_count..=max_count)
        .map(|length| -> syn::Result<TokenStream> {
            let self_tuple = TupleInfo {
                length,
                prefix: "_".to_string(),
            };

            let mut modifier = CodeModifier {
                self_tuple: self_tuple.clone(),
                tuples: BTreeMap::new(),
                constants: Default::default(),
            };

            modifier
                .tuples
                .insert(Ident::new("Self", input.self_ty.span()), self_tuple);
            for (name, length_expr) in &additional_tuples {
                let length = eval_index(length_expr, Some(&modifier))?;
                modifier.tuples.insert(
                    name.clone(),
                    TupleInfo {
                        length,
                        prefix: format!("_{name}"),
                    },
                );
            }

            let mut item = input.clone();

            modifier.visit_item_impl_mut(&mut item);

            Ok(item.into_token_stream())
        })
        .collect()
}

fn as_tuple_bound(bound: &TypeParamBound) -> syn::Result<Option<Expr>> {
    if let TypeParamBound::Trait(TraitBound { path, .. }) = &bound
        && path.segments.len() == 1
        && path.segments[0].ident == "Tuple"
    {
        let PathArguments::AngleBracketed(args) = &path.segments[0].arguments else {
            return Err(syn::Error::new(
                path.segments[0].arguments.span(),
                "Tuple type has a required argument",
            ));
        };
        if args.args.is_empty() {
            return Err(syn::Error::new(
                path.segments[0].arguments.span(),
                "Tuple type has a required argument",
            ));
        }
        if args.args.len() > 1 {
            return Err(syn::Error::new(
                path.segments[0].arguments.span(),
                "Tuple type only has a single generic argument",
            ));
        }
        let GenericArgument::Const(length) = &args.args[0] else {
            return Err(syn::Error::new(
                args.args[0].span(),
                "Tuple expects a const generic arg (with braces {...})",
            ));
        };
        Ok(Some(length.clone()))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct TupleInfo {
    length: usize,
    prefix: String,
}

impl TupleInfo {
    fn ident(&self, index: usize) -> Ident {
        format_ident!("{}{}", &self.prefix, index)
    }

    fn ty(&self, index: usize) -> Type {
        Type::Path(TypePath {
            attrs: vec![],
            qself: None,
            path: self.ident(index).into(),
        })
    }

    fn idents(&self) -> impl Iterator<Item = Ident> {
        (0..self.length).map(|i| self.ident(i))
    }
    fn types(&self) -> impl Iterator<Item = Type> {
        self.idents().map(|ident| {
            Type::Path(TypePath {
                attrs: vec![],
                qself: None,
                path: ident.into(),
            })
        })
    }
    fn ident_range_i(&self, range: IndexRange) -> impl Iterator<Item = (usize, Ident)> {
        range
            .into_iter(self.length)
            .map(|i| (i, format_ident!("{}{}", &self.prefix, i)))
    }
    fn type_range_i(&self, range: IndexRange) -> impl Iterator<Item = (usize, Type)> {
        self.ident_range_i(range).map(|(i, ident)| {
            (
                i,
                Type::Path(TypePath {
                    attrs: vec![],
                    qself: None,
                    path: ident.into(),
                }),
            )
        })
    }
}

struct CodeModifier {
    self_tuple: TupleInfo,
    tuples: BTreeMap<Ident, TupleInfo>,
    constants: BTreeMap<Ident, usize>,
}

impl VisitMut for CodeModifier {
    fn visit_item_impl_mut(&mut self, item_impl: &mut ItemImpl) {
        *item_impl.self_ty = Type::Tuple(TypeTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: self.self_tuple.types().collect(),
        });
        for tuple in self.tuples.values() {
            for i in tuple.idents() {
                item_impl
                    .generics
                    .params
                    .push(GenericParam::Type(TypeParam {
                        attrs: vec![],
                        ident: i,
                        colon_token: None,
                        bounds: Default::default(),
                        default: None,
                    }))
            }
        }
        visit_mut::visit_item_impl_mut(self, item_impl);
    }

    fn visit_where_clause_mut(&mut self, clause: &mut WhereClause) {
        let predicates = mem::take(&mut clause.predicates);
        clause.predicates = predicates
            .into_iter()
            .flat_map(|predicate| {
                self.modify_where_predicates(predicate)
                    .unwrap_or_else(|error| {
                        vec![WherePredicate::Type(PredicateType {
                            attrs: vec![],
                            lifetimes: None,
                            bounded_ty: syn::parse2(error.into_compile_error()).unwrap(),
                            colon_token: Default::default(),
                            bounds: Default::default(),
                        })]
                    })
            })
            .collect();
    }

    fn visit_local_mut(&mut self, local: &mut Local) {
        if let Err(error) = self.inner_visit_local_mut(local) {
            local.init = Some(LocalInit {
                eq_token: Default::default(),
                expr: parse2(error.into_compile_error()).unwrap(),
                diverge: None,
            })
        }
        visit_mut::visit_local_mut(self, local)
    }

    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Err(error) = self.inner_visit_expr_mut(expr) {
            *expr = parse2(error.into_compile_error()).unwrap()
        }
        visit_mut::visit_expr_mut(self, expr)
    }

    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Err(error) = self.inner_visit_type_mut(ty) {
            *ty = parse2(error.into_compile_error()).unwrap()
        }
        visit_mut::visit_type_mut(self, ty)
    }
}

impl CodeModifier {
    fn modify_where_predicates(
        &mut self,
        predicate: WherePredicate,
    ) -> syn::Result<Vec<WherePredicate>> {
        if let WherePredicate::Type(predicate) = &predicate
            && let Type::Path(ty) = &predicate.bounded_ty
            && ty.path.segments.len() == 2
            && let Some(tuple) = self.tuples.get(&ty.path.segments[0].ident)
            && ty.path.segments[0].arguments == PathArguments::None
            && ty.path.segments[1].ident == "Element"
            && let PathArguments::AngleBracketed(args) = &ty.path.segments[1].arguments
            && args.args.len() == 1
            && let GenericArgument::Const(range) = &args.args[0]
        {
            let Expr::Block(ExprBlock {
                block: Block { stmts, .. },
                ..
            }) = &range
            else {
                return Err(syn::Error::new(
                    range.span(),
                    "unexpected expression, expected block",
                ));
            };
            if stmts.len() != 1 {
                return Err(syn::Error::new(
                    range.span(),
                    "unexpected expression, expected block with a single expression",
                ));
            }
            let Stmt::Expr(range, None) = &stmts[0] else {
                return Err(syn::Error::new(
                    range.span(),
                    "unexpected expression, expected block with a single expression",
                ));
            };

            let (range, binding) = if let Expr::Assign(ExprAssign {
                left: binding,
                right: range,
                ..
            }) = range
            {
                let binding: Ident = syn::parse2(binding.to_token_stream())
                    .with_message("cannot parse index binding")?;
                (&**range, Some(binding))
            } else {
                (range, None)
            };
            println!(
                "range: {}, binding = {:?}",
                range.to_token_stream(),
                binding.as_ref().map(|b| b.to_token_stream().to_string())
            );
            let range = eval_range(range, Some(self)).with_message("invalid range")?;
            Ok(tuple
                .clone()
                .type_range_i(range)
                .map(|(i, ty)| {
                    let mut predicate = predicate.clone();
                    predicate.bounded_ty = ty;
                    if let Some(binding) = &binding {
                        self.constants.insert(binding.clone(), i);
                        self.visit_predicate_type_mut(&mut predicate);
                        self.constants.remove(binding);
                    }
                    WherePredicate::Type(predicate)
                })
                .collect::<Vec<_>>())
        } else {
            Ok(vec![predicate])
        }
    }

    fn inner_visit_local_mut(&mut self, local: &mut Local) -> syn::Result<()> {
        let Some(unroll_loop) = get_unroll_loop(&mut local.attrs) else {
            return Ok(());
        };
        let Some(init) = &mut local.init else {
            return Err(syn::Error::new(
                unroll_loop,
                "#[unroll_loop] cannot be used on unassigned let bindings",
            ));
        };
        let expr_span = init.expr.span();
        let Expr::ForLoop(for_loop) = &*init.expr else {
            return Err(syn::Error::new(
                expr_span,
                "expected for loop due to #[unroll_loop]",
            ));
        };
        *init.expr = self.unroll_for_loop(for_loop)?;
        Ok(())
    }

    fn inner_visit_expr_mut(&mut self, expr: &mut Expr) -> syn::Result<()> {
        'mutate: {
            match expr {
                Expr::ForLoop(for_loop) => {
                    if get_unroll_loop(&mut for_loop.attrs).is_some() {
                        *expr = self.unroll_for_loop(for_loop)?
                    }
                }
                Expr::Macro(expr_macro) => {
                    if expr_macro.mac.path.is_ident("__tuple_index") {
                        let parse = |input: ParseStream| -> syn::Result<(Expr, usize)> {
                            let content;
                            parenthesized!(content in input);
                            let expr = content.parse::<Expr>()?;
                            let _: Token![,] = input.parse()?;
                            let content;
                            bracketed!(content in input);
                            let mut index: Expr = content.parse()?;
                            self.visit_expr_mut(&mut index);
                            let index = eval_index(&index, Some(self))?;
                            Ok((expr, index))
                        };
                        let tokens = mem::take(&mut expr_macro.mac.tokens);
                        let (receiver, index) = parse.parse2(tokens)?;
                        *expr = Expr::Field(ExprField {
                            attrs: vec![],
                            base: Box::new(receiver),
                            dot_token: Default::default(),
                            member: Member::Unnamed(index.into()),
                        })
                    }
                }

                Expr::Path(expr_path) => {
                    let Some(ident) = expr_path.path.get_ident() else {
                        break 'mutate;
                    };
                    let Some(value) = self.constants.get(ident) else {
                        break 'mutate;
                    };
                    *expr = Expr::Lit(ExprLit {
                        attrs: vec![],
                        lit: Lit::Int(LitInt::new(
                            &value.to_string(),
                            expr_path.path.segments[0].ident.span(),
                        )),
                    })
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn unroll_for_loop(&mut self, for_loop: &ExprForLoop) -> syn::Result<Expr> {
        let index: Ident = parse2(for_loop.pat.to_token_stream())?;
        let range = eval_range(&for_loop.expr, Some(self))?;

        Ok(Expr::Tuple(ExprTuple {
            attrs: vec![],
            paren_token: Default::default(),
            elems: range
                .into_iter_require_end()?
                .map(move |i| {
                    self.constants.insert(index.clone(), i);
                    let mut block = for_loop.body.clone();
                    self.visit_block_mut(&mut block);
                    self.constants.remove(&index);
                    Expr::Block(ExprBlock {
                        attrs: vec![],
                        label: None,
                        block,
                    })
                })
                .collect(),
        }))
    }

    fn inner_visit_type_mut(&mut self, ty: &mut Type) -> syn::Result<()> {
        let debug = ty.to_token_stream().to_string();
        match ty {
            Type::Path(type_path)
                if type_path.path.segments.len() == 2
                    && let Some(tuple) = self.tuples.get(&type_path.path.segments[0].ident)
                    && type_path.path.segments[0].arguments == PathArguments::None
                    && type_path.path.segments[1].ident == "Element"
                    && let PathArguments::AngleBracketed(args) =
                        &type_path.path.segments[1].arguments
                    && args.args.len() == 1
                    && let GenericArgument::Const(index) = &args.args[0] =>
            {
                let index = eval_index(index, Some(self))?;
                *ty = tuple.ty(index)
            }
            Type::Path(type_path)
                if type_path.path.segments.len() == 2
                    && let Some(tuple) = self.tuples.get(&type_path.path.segments[0].ident)
                    && type_path.path.segments[0].arguments == PathArguments::None
                    && type_path.path.segments[1].ident == "Mapped"
                    && let PathArguments::AngleBracketed(args) =
                        &type_path.path.segments[1].arguments
                    && args.args.len() == 2
                    && let GenericArgument::Type(t) = &args.args[0]
                    && let Type::Path(TypePath { path, .. }) = t
                    && let Some(t) = path.get_ident()
                    && let GenericArgument::Type(pat) = &args.args[1] =>
            {
                *ty = Type::Tuple(TypeTuple {
                    attrs: vec![],
                    paren_token: Default::default(),
                    elems: tuple
                        .idents()
                        .map(|i| {
                            let mut visitor = TypeReplacer {
                                find: t.clone(),
                                replace: i,
                            };
                            let mut element_ty = pat.clone();
                            visitor.visit_type_mut(&mut element_ty);
                            element_ty
                        })
                        .collect(),
                })
            }
            Type::Path(type_path)
                if let Some(ident) = type_path.path.get_ident()
                    && let Some(tuple) = self.tuples.get(ident) =>
            {
                *ty = Type::Tuple(TypeTuple {
                    attrs: vec![],
                    paren_token: Default::default(),
                    elems: tuple.types().collect(),
                })
            }
            _ => {}
        }
        drop(debug);
        Ok(())
    }
}

struct TypeReplacer {
    find: Ident,
    replace: Ident,
}

impl VisitMut for TypeReplacer {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(TypePath { path, .. }) = ty
            && let PathSegment {
                ident,
                arguments: PathArguments::None,
            } = &mut path.segments[0]
            && ident == &self.find
        {
            *ident = self.replace.clone();
        }
        visit_mut::visit_type_mut(self, ty)
    }
}

fn get_unroll_loop(attrs: &mut Vec<Attribute>) -> Option<Span> {
    let owned = mem::take(attrs);
    let mut span = None;
    *attrs = owned
        .into_iter()
        .filter(|attr| {
            if let Meta::Path(_) = &attr.meta
                && attr.path().is_ident("unroll_loop")
            {
                span = Some(attr.span());
                false
            } else {
                true
            }
        })
        .collect();
    span
}

struct IndexRange {
    span: Span,
    // inclusive
    start: Option<usize>,
    // exclusive
    end: Option<usize>,
}

impl IndexRange {
    fn into_iter(self, length: usize) -> impl Iterator<Item = usize> {
        let start = self.start.unwrap_or(0);
        let end = self.end.unwrap_or(length);
        start..end
    }
    fn into_iter_require_end(self) -> syn::Result<impl Iterator<Item = usize>> {
        let Some(end) = self.end else {
            return Err(syn::Error::new(
                self.span,
                "open ended range is not allowed here",
            ));
        };
        let start = self.start.unwrap_or(0);
        Ok(start..end)
    }
}

fn eval_index(input: &Expr, modifier: Option<&CodeModifier>) -> syn::Result<usize> {
    match input {
        Expr::Binary(expr) => {
            let left = eval_index(&expr.left, modifier)?;
            let right = eval_index(&expr.right, modifier)?;
            match expr.op {
                BinOp::Add(_) => {
                    let value = left + right;
                    Ok(value)
                }
                BinOp::Sub(_) => {
                    let value = left - right;
                    Ok(value)
                }
                BinOp::Mul(_) => {
                    let value = left * right;
                    Ok(value)
                }
                BinOp::Div(_) => {
                    let value = left / right;
                    Ok(value)
                }
                BinOp::Rem(_) => {
                    let value = left % right;
                    Ok(value)
                }
                BinOp::BitXor(_) => {
                    let value = left ^ right;
                    Ok(value)
                }
                BinOp::BitAnd(_) => {
                    let value = left & right;
                    Ok(value)
                }
                BinOp::BitOr(_) => {
                    let value = left | right;
                    Ok(value)
                }
                BinOp::Shl(_) => {
                    let value = left << right;
                    Ok(value)
                }
                BinOp::Shr(_) => {
                    let value = left >> right;
                    Ok(value)
                }
                op => {
                    let op = op.to_token_stream().to_string();
                    Err(syn::Error::new(
                        op.span(),
                        format!("binary operator {op} is not supported in this context"),
                    ))
                }
            }
        }
        Expr::Group(expr) => eval_index(&expr.expr, modifier),
        Expr::Lit(expr) => match &expr.lit {
            Lit::Int(int) => Ok(int.base10_parse()?),
            lit => {
                let lit = lit.to_token_stream().to_string();
                Err(syn::Error::new(
                    lit.span(),
                    format!("only integer literals are supported in this context, found: {lit}"),
                ))
            }
        },
        Expr::Paren(expr) => eval_index(&expr.expr, modifier),
        Expr::Block(expr) => {
            if expr.block.stmts.len() != 1 {
                return Err(syn::Error::new(
                    expr.span(),
                    "Block must have exactly one expression",
                ));
            }
            let stmt = &expr.block.stmts[0];
            let Stmt::Expr(expr, None) = &stmt else {
                return Err(syn::Error::new(
                    stmt.span(),
                    "expression expected, found statement",
                ));
            };
            eval_index(expr, modifier)
        }
        expr => {
            if let Some(modifier) = modifier
                && let Expr::Path(expr) = expr
                && expr.path.segments.len() == 2
                && let Some(tuple) = modifier.tuples.get(&expr.path.segments[0].ident)
                && expr.path.segments[0].arguments == PathArguments::None
                && expr.path.segments[1].ident == "Length"
                && expr.path.segments[1].arguments == PathArguments::None
            {
                Ok(tuple.length)
            } else if let Some(modifier) = modifier
                && let Expr::Path(expr) = expr
                && let Some(ident) = expr.path.get_ident()
                && let Some(value) = modifier.constants.get(ident)
            {
                Ok(*value)
            } else {
                let expr_tokens = expr.to_token_stream().to_string();
                Err(syn::Error::new(
                    expr.span(),
                    format!(
                        "expression kind not supported in this context: {expr_tokens}, {expr:?}"
                    ),
                ))
            }
        }
    }
}

fn eval_range(input: &Expr, tuples: Option<&CodeModifier>) -> syn::Result<IndexRange> {
    match input {
        Expr::Range(expr) => {
            let span = expr.span();
            let start = if let Some(start) = &expr.start {
                let value = eval_index(start, tuples)?;
                Some(value)
            } else {
                None
            };
            let mut end = if let Some(end) = &expr.end {
                let value = eval_index(end, tuples)?;
                Some(value)
            } else {
                None
            };
            let inclusive = matches!(expr.limits, RangeLimits::Closed(..));
            if inclusive && let Some(value) = end {
                end = Some(value + 1)
            }
            Ok(IndexRange { span, start, end })
        }
        Expr::Block(expr) => {
            if expr.block.stmts.len() != 1 {
                return Err(syn::Error::new(
                    expr.span(),
                    "Block must have exactly one expression",
                ));
            }
            let stmt = &expr.block.stmts[0];
            let Stmt::Expr(expr, None) = &stmt else {
                return Err(syn::Error::new(
                    stmt.span(),
                    "expression expected, found statement",
                ));
            };
            eval_range(expr, tuples)
        }
        expr => {
            let expr_tokens = expr.to_token_stream().to_string();
            Err(syn::Error::new(
                expr.span(),
                format!("expression kind not supported in this context: {expr_tokens}, {expr:?}"),
            ))
        }
    }
}

trait WithMessage {
    fn with_message(self, msg: &str) -> Self;
}

impl<T> WithMessage for syn::Result<T> {
    #[cfg(test)]
    fn with_message(self, msg: &str) -> Self {
        self.map_err(|err| syn::Error::new(err.span(), format!("{msg}: {err}")))
    }
    #[cfg(not(test))]
    fn with_message(self, _msg: &str) -> Self {
        self
    }
}
