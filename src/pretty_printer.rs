use crate::compiler_context::CompilerContext;

pub fn pretty<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> String {
    let (res, _might_need_parens) = pretty_impl(context, id);
    res
}

fn pretty_inner<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> String {
    let (res, might_need_parens) = pretty_impl(context, id);
    if might_need_parens {
        format!("({})", res)
    } else {
        res
    }
}

pub fn pretty_impl<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> (String, bool) {
    if let Ok(v) = context.get_i64(id) {
        return (format!("{}", v), *v<0);
    }
    if let Ok(s) = context.get_symbol(id) {
        return (s.name.to_string(), false);
    }
    if let Ok(c) = context.get_call(id) {
        let callee = pretty_inner(context, c.callee);
        let mut is_operator_call = if let Ok(sym) = context.get_symbol(c.callee) {
            sym.is_operator
        } else {
            false
        };
        let mut arg_num = 0;
        let args: Vec<String> = {
            let is_operator_call = &mut is_operator_call;
            c
            .args
            .iter()
            .map(|(name, arg)| {
                if name.starts_with("arg_") && name == &format!("arg_{}", arg_num) {
                    arg_num += 1;
                    pretty_inner(context, *arg)
                } else {
                    *is_operator_call = false;
                    format!("{}={}", name, pretty(context, *arg))
                }
            })
            .collect()
        };
        if is_operator_call {
            let args = args.join(&callee);
            return (format!("{}{}", if c.args.len() < 2 { &callee } else { "" }, args), true);
        }
        let args = args.join(", ");
        return (format!("{}({})", callee, args), false);
    }
    (format!("{{node? {:?}}}", id), false)
}
