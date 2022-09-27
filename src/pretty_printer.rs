use crate::compiler_context::CompilerContext;

pub fn pretty<C: CompilerContext + ?Sized>(context: &C, id: C::ID) -> String {
    if let Ok(v) = context.get_i64(id) {
        if *v < 0 {
            return format!("({})", v);
        }
        return format!("{}", v);
    }
    if let Ok(s) = context.get_symbol(id) {
        return s.name.to_string();
    }
    if let Ok(c) = context.get_call(id) {
        let callee = context.pretty(c.callee);
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
                    context.pretty(*arg)
                } else {
                    *is_operator_call = false;
                    format!("{}={}", name, context.pretty(*arg))
                }
            })
            .collect()
        };
        if is_operator_call {
            let args = args.join(&callee);
            return format!("({}{})", if c.args.len() < 2 { &callee } else { "" }, args);
        }
        let args = args.join(", ");
        return format!("{}({})", callee, args);
    }
    format!("{{node? {:?}}}", id)
}
