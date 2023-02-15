pub fn generate_imports(scope_block: String) -> Result<String, &'static str> {
    if !scope_block.is_empty() {
        return Err("imports should be at the start of a new empty scope block");
    }
    let new_scope_block = scope_block + "use std::marker::PhantomData;\n";
    Ok(new_scope_block)
}

pub fn generate_st_traits(scope_block: String) -> Result<String, &'static str> {
    let mut new_scope_block = scope_block
        + "\n"
        + &format!("pub trait Action {{\n\tfn new() -> Self where Self:Sized;\n}}\n");
    new_scope_block += "\npub trait Message {}\n";
    Ok(new_scope_block)
}

pub fn generate_offer(
    role: &str,
    scope_block: String,
    num_choices: u32,
) -> Result<String, &'static str> {
    let struct_name = &format!("OfferToRole{}{}Choices", role, num_choices);
    let mut params = "<".to_owned();
    for num in 0..num_choices {
        params += &format!("M{}, ", num);
    }
    for num in 0..num_choices - 1 {
        params += &format!("A{}, ", num);
    }
    params += &format!("A{}>", num_choices - 1);
    let mut where_clauses = "\nwhere\n".to_owned();
    for num in 0..num_choices {
        where_clauses += &format!("\tM{}: Message,\n", num);
    }
    for num in 0..num_choices {
        where_clauses += &format!("\tA{}: Action,\n", num)
    }

    let mut new_scope_block = scope_block + "\n" + &format!("enum Branchings{} {{\n", struct_name);
    for num in 0..num_choices {
        new_scope_block += &format!("\tBranch{},\n", num);
    }
    new_scope_block += "}\n";
    new_scope_block += &format!("\nstruct {}", struct_name);
    new_scope_block += &params;
    new_scope_block += &where_clauses;
    new_scope_block += "{\n\tphantom: PhantomData<(";
    for num in 0..num_choices {
        new_scope_block += &format!("M{}, ", num);
    }
    for num in 0..num_choices - 1 {
        new_scope_block += &format!("A{}, ", num);
    }
    new_scope_block += &format!("A{}),>\n}}\n", num_choices - 1);
    new_scope_block += "\nimpl";
    new_scope_block += &params;
    new_scope_block += &format!("Action for {}", struct_name);
    new_scope_block += &params;
    new_scope_block += &where_clauses;
    new_scope_block += &format!(
        "{{\n\tfn new() -> Self {{\n\t\t{} {{\n\t\t\tphantom: PhantomData,\n\t\t}}\n\t}}\n}}\n",
        struct_name
    );
    new_scope_block += "\nimpl";
    new_scope_block += &params;
    new_scope_block += &format!("{}", struct_name);
    new_scope_block += &params;
    new_scope_block += "\nwhere\n";
    for num in 0..num_choices {
        new_scope_block += &format!("\tM{}: Message + 'static,\n", num);
    }
    for num in 0..num_choices {
        new_scope_block += &format!("\tA{}: Action + 'static,\n", num)
    }
    new_scope_block += "\n{";
    new_scope_block += "\n\tpub fn offer(self,";
    for num in 0..num_choices {
        new_scope_block += &format!("emitter{}: Box<dyn Fn() -> M{}>,", num, num);
    }
    new_scope_block += &format!(
        "picker: Box<dyn Fn() -> Branchings{}>) -> (Box<dyn Message>, Box<dyn Action>) {{\n",
        struct_name
    );
    let mut branch_match_arms = "".to_owned();
    for num in 0..num_choices {
        branch_match_arms += &format!("\t\tBranchings{}::Branch{} => {{\n\t\t\tlet message = emitter{}();\n\t\t\treturn (Box::new(message), Box::new(A{}::new()));\n\t\t}}\n",
            struct_name, num, num, num);
    }
    new_scope_block += "\tlet choice = picker();\n";
    new_scope_block += "\tmatch choice{\n";
    new_scope_block += &branch_match_arms;
    new_scope_block += "\t}\n}\n}";
    Ok(new_scope_block)
}

pub fn generate_selection(
    role: &str,
    scope_block: String,
    num_choices: u32,
) -> Result<String, &'static str> {
    let struct_name = &format!("SelectFromRole{}{}Choices", role, num_choices);
    let mut params = "<".to_owned();
    for num in 0..num_choices {
        params += &format!("M{}, ", num);
    }
    for num in 0..num_choices - 1 {
        params += &format!("A{}, ", num);
    }
    params += &format!("A{}>", num_choices - 1);
    let mut where_clauses = "\nwhere\n".to_owned();
    for num in 0..num_choices {
        where_clauses += &format!("\tM{}: Message,\n", num);
    }
    for num in 0..num_choices {
        where_clauses += &format!("\tA{}: Action,\n", num)
    }
    let mut new_scope_block = scope_block + "\n" + &format!("enum Branchings{} {{\n", struct_name);
    for num in 0..num_choices {
        new_scope_block += &format!("\tBranch{},\n", num);
    }
    new_scope_block += "}\n";
    new_scope_block += &format!("\nstruct {}", struct_name);
    new_scope_block += &params;
    new_scope_block += &where_clauses;
    new_scope_block += "{\n\tphantom: PhantomData<(";
    for num in 0..num_choices {
        new_scope_block += &format!("M{}, ", num);
    }
    for num in 0..num_choices - 1 {
        new_scope_block += &format!("A{}, ", num);
    }
    new_scope_block += &format!("A{}),>\n}}\n", num_choices - 1);
    new_scope_block += "\nimpl";
    new_scope_block += &params;
    new_scope_block += &format!("Action for {}", struct_name);
    new_scope_block += &params;
    new_scope_block += &where_clauses;
    new_scope_block += &format!(
        "{{\n\tfn new() -> Self {{\n\t\t{} {{\n\t\t\tphantom: PhantomData,\n\t\t}}\n\t}}\n}}\n",
        struct_name
    );
    new_scope_block += "\nimpl";
    new_scope_block += &params;
    new_scope_block += &format!("{}", struct_name);
    new_scope_block += &params;
    new_scope_block += "\nwhere\n";
    for num in 0..num_choices {
        new_scope_block += &format!("\tM{}: Message + 'static,\n", num);
    }
    for num in 0..num_choices {
        new_scope_block += &format!("\tA{}: Action + 'static,\n", num)
    }
    new_scope_block += "\n{";
    new_scope_block += "\n\tpub fn select(self,";
    for num in 0..num_choices {
        new_scope_block += &format!("emitter{}: Box<dyn Fn(M{})>, ", num, num);
    }
    for num in 0..num_choices {
        new_scope_block += &format!("message{}: M{}, ", num, num);
    }
    new_scope_block += &format!(
        "picker: Box<dyn Fn() -> Branchings{}>) -> Box<dyn Action> {{\n",
        struct_name
    );
    let mut branch_match_arms = "".to_owned();
    for num in 0..num_choices {
        branch_match_arms += &format!("\t\tBranchings{}::Branch{} => {{\n\t\t\temitter{}(message{});\n\t\t\treturn Box::new(A{}::new());\n\t\t}}\n",
            struct_name, num, num, num, num);
    }
    new_scope_block += "\tlet choice = picker();\n";
    new_scope_block += "\tmatch choice{\n";
    new_scope_block += &branch_match_arms;
    new_scope_block += "\t}\n}\n}";
    Ok(new_scope_block)
}

pub fn generate_end(scope_block: String) -> Result<String, &'static str> {
    let new_scope_block = scope_block
        + "\n"
        + &format!("struct End {{}}\n")
        + "\n"
        + &format!("impl Action for End {{\n\tfn new() -> Self {{\n\t\tEnd {{}}\n\t}}\n}}\n");
    Ok(new_scope_block)
}

#[cfg(test)]
mod tests {
    use crate::{
        generate_end, generate_imports, generate_offer, generate_selection, generate_st_traits,
    };

    #[test]
    fn it_works() {
        let scope = "".to_owned();
        let scope = generate_imports(scope).unwrap();
        let scope = generate_st_traits(scope).unwrap();
        let scope = generate_offer("A", scope, 3).unwrap();
        let scope = generate_selection("A", scope, 3).unwrap();
        let scope = generate_end(scope).unwrap();
        eprintln!("{}", scope);
    }
}

mod example;
