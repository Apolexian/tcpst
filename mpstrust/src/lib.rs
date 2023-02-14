pub fn generate_imports(scope_block: String) -> Result<String, &'static str> {
    if !scope_block.is_empty() {
        return Err("imports should be at the start of a new empty scope block");
    }
    let new_scope_block = scope_block + "use std::marker::PhantomData;\n";
    Ok(new_scope_block)
}

pub fn generate_st_traits(scope_block: String) -> Result<String, &'static str> {
    let new_scope_block =
        scope_block + "\n" + &format!("pub trait Action {{\n\tfn new() -> Self;\n}}\n");
    Ok(new_scope_block)
}

pub fn generate_actions_for_role(role: &str, scope_block: String) -> Result<String, &'static str> {
    let new_scope_block = scope_block + 
        "\n" + 
        &format!("struct RecvFromRole{}<M, A>\nwhere\n\tA: Action,\n{{\n\tphantom: PhantomData<(M, A)>,\n}}\n", role);
    let new_scope_block = new_scope_block +
        "\n" + 
        &format!("impl<M, A> Action for RecvFromRole{}<M, A>\nwhere\n\tA: Action,\n{{\n\tfn new() -> Self {{\n\t\tRecvFromRole{} {{\n\t\t\tphantom: PhantomData,\n\t\t}}\n\t}}\n}}\n", role, role);
    let new_scope_block = new_scope_block +
        "\n" +
        &format!("impl<M, A> RecvFromRole{}<M, A>\nwhere\n\tA: Action,\n{{\n\tpub fn recv(self, emitter: Box<dyn Fn() -> M>) -> (M, A) {{\n\t\tlet message = emitter();\n\t\t(message, A::new())\n\t}}\n}}\n", role);
    let new_scope_block = new_scope_block +
        "\n" +
        &format!("struct SendToRole{}<M, A>\nwhere\n\tA: Action,\n{{\n\tphantom: PhantomData<(M, A)>,\n}}\n", role);
    let new_scope_block = new_scope_block +
        "\n" +
        &format!("impl<M, A> SendToRole{}<M, A>\nwhere\n\tA: Action,\n{{\n\tpub fn send(self, message: M, emitter: Box<dyn Fn(M)>) -> A {{\n\t\temitter(message);\n\t\tA::new()\n\t}}\n}}\n", role);
    let new_scope_block = new_scope_block +
        "\n" +
        &format!(
        "impl<M, A> Action for SendToRole{}<M, A>\nwhere\n\tA: Action,\n{{\n\tfn new() -> Self {{\n\t\tSendToRole{} {{\n\t\t\tphantom: PhantomData,\n\t\t}}\n\t}}\n}}\n", role, role);
    Ok(new_scope_block)
}

pub fn generate_end(scope_block: String) -> Result<String, &'static str> {
    let new_scope_block = scope_block + 
        "\n" + 
        &format!("struct End {{}}\n") +
        "\n" +
        &format!("impl Action for End {{\n\tfn new() -> Self {{\n\t\tEnd {{}}\n\t}}\n}}\n");
    Ok(new_scope_block)
}

#[cfg(test)]
mod tests {
    use crate::{generate_actions_for_role, generate_imports, generate_st_traits, generate_end};

    #[test]
    fn it_works() {
        let scope = "".to_owned();
        let scope = generate_imports(scope).unwrap();
        let scope = generate_st_traits(scope).unwrap();
        let scope = generate_actions_for_role("A", scope).unwrap();
        let scope = generate_actions_for_role("B", scope).unwrap();
        let scope = generate_actions_for_role("C", scope).unwrap();
        let scope = generate_end(scope).unwrap();
        eprintln!("{}", scope);
    }
}

mod example;
