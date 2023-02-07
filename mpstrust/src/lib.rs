pub fn generate_imports(scope_block: String) -> Result<String, &'static str> {
    if !scope_block.is_empty() {
        return Err("imports should be at the start of a new empty scope block");
    }
    let new_scope_block = scope_block + "use std::marker::{Sized, Send, PhantomData};\n";
    let new_scope_block = new_scope_block + "use std::crossbeam_channel::{Sender, Receiver};\n";
    Ok(new_scope_block)
}

pub fn generate_st_traits(scope_block: String) -> Result<String, &'static str> {
    let new_scope_block = scope_block + "\n" + &format!("pub trait Action {{}}\n");
    let new_scope_block = new_scope_block
        + "\n"
        + &format!(
            "pub trait SendEndpoint<M>: std::marker::Send {{\n\tfn send(&self, message: M);\n}}\n"
        );
    let new_scope_block = new_scope_block
        + "\n"
        + &format!("pub trait RecvEndpoint<M>: std::marker::Send {{\n\tfn recv(&self) -> M;\n}}\n");
    let new_scope_block = new_scope_block
        + "\n"
        + &format!(
            "pub struct Snd<M, A>\nwhere\n\tA:Action,\n{{\n\tphantom: PhantomData<(M,A)>,\n}}\n"
        );
    let new_scope_block = new_scope_block
        + "\n"
        + &format!("impl<M, A> Action for Snd<M, A>\nwhere\n\tA:Action,\n{{}}\n");
    let new_scope_block = new_scope_block
        + "\n"
        + &format!(
            "pub struct Recv<M, A>\nwhere\n\tA:Action,\n{{\n\tphantom: PhantomData<(M,A)>,\n}}\n"
        );
    let new_scope_block = new_scope_block
        + "\n"
        + &format!("impl<M, A> Action for Recv<M,A>\nwhere\n\tA:Action,\n{{}}\n");
    Ok(new_scope_block)
}

pub fn generate_channel(
    role_from: &str,
    role_to: &str,
    scope_block: String,
) -> Result<String, &'static str> {
    let strcut_name = &format!("Channel{}{}<A, M>", role_from, role_to);
    let sender_field = "sender: Box<dyn SendEndpoint<M>>,";
    let recv_field = "receiver: Box<dyn RecvEndpoint<M>>,";
    let phantom = "phantom: PhantomData<A>";
    let block = format!(
        "struct {}\nwhere\n\tA:Action,\n{{\n\t{}\n\t{}\n\t{}\n}}",
        strcut_name, sender_field, recv_field, phantom
    );
    let new_scope = scope_block + "\n" + &block;
    Ok(new_scope)
}

#[cfg(test)]
mod tests {
    use crate::{generate_channel, generate_imports, generate_st_traits};

    #[test]
    fn it_works() {
        let scope = "".to_owned();
        let scope = generate_imports(scope).unwrap();
        let scope = generate_st_traits(scope).unwrap();
        let scope = generate_channel("A", "B", scope).unwrap();
        eprintln!("{}", scope);
    }
}
