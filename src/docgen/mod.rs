use crate::checker::mdir::MiddleIR;

pub fn gen_docs(mdir: MiddleIR) -> String {
    let mut result = String::new();

    mdir.functions().iter().for_each(|(_, f)| {
        result += &f.name;
        result += "(";
        for (name, ty) in &f.params {
            result += &format!("{name} : {:?}", ty)
        }
        result += ")";
        result += &format!("{:?} \n\n", &f.return_type);

        // TODO: Report parse errors and such
        for dc in &f.doc_comments {
            result += &dc.md;
        }
    });

    result
}