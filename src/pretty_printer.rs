use crate::compiler_context::CompilerContext;

pub fn pretty<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> String {
    let (res, _complex_expr, is_operator) = pretty_impl(context, id);
    if is_operator {
        format!("({})", res)
    } else {
        res
    }
}

fn pretty_inner<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> String {
    let (res, complex_expr, _is_operator) = pretty_impl(context, id);
    if complex_expr {
        format!("({})", res)
    } else {
        res
    }
}

pub fn pretty_impl<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> (String, bool, bool) {
    if let Ok(v) = context.get_i64(id) {
        return (format!("{}", v), *v < 0, false);
    }
    if let Ok(s) = context.get_symbol(id) {
        return (s.name.to_string(), false, s.is_operator);
    }
    if let Ok(c) = context.get_call(id) {
        let (mut callee, is_expr, is_op) = pretty_impl(context, c.callee);
        let mut attempt_operator_call = true;
        let mut arg_num = 0;
        let args: Vec<String> = {
            let attempt_operator_call = &mut attempt_operator_call;
            c.args
                .iter()
                .map(|(name, arg)| {
                    if name.starts_with("arg_") && name == &format!("arg_{}", arg_num) {
                        arg_num += 1;
                        pretty_inner(context, *arg)
                    } else {
                        *attempt_operator_call = false;
                        format!("{}={}", name, pretty(context, *arg))
                    }
                })
                .collect()
        };
        if is_op && !is_expr && attempt_operator_call {
            let args = args.join(&callee);
            return (
                format!("{}{}", if c.args.len() < 2 { &callee } else { "" }, args),
                true,
                false
            );
        }
        if is_expr || is_op {
            callee = format!("({})", callee);
        }
        let args = args.join(", ");
        return (format!("{}({})", callee, args), false, false);
    }
    (format!("{{node? {:?}}}", id), false, true)
}
