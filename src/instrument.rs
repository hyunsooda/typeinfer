pub fn gen_assert(typ1: &str, typ2: &str) -> String {
    format!(
        "if (typeof({}) !== typeof({})) throw `type assertion failed ${{{}}} !== ${{{}}}`",
        typ1, typ2, typ1, typ2
    )
}
