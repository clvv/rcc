use num_bigint::BigUint;
use polyexen::plaf::Plaf;

fn maybe_biguint_to_str(u: &Option<BigUint>) -> String {
    if let Some(n) = u {
        let digits: Vec<u32> = n.iter_u32_digits().map(|i| i.into()).collect();
        if digits.len() > 0 {
            format!("{:?}", digits)
        } else {
            String::from("[0]")
        }
    } else {
        String::from("[0]")
    }
}

fn plaf_fixed_to_toml(fixed: Vec<Vec<Option<BigUint>>>) -> String {
    let mut toml_string = String::new();
    toml_string.push_str("fixed = [\n");
    for (i, column) in fixed.iter().enumerate() {
        toml_string.push_str("  [");
        for (j, value) in column.iter().enumerate() {
            toml_string.push_str(&maybe_biguint_to_str(value));
            if j != column.len() - 1 {
                toml_string.push_str(", ")
            }
        }
        toml_string.push_str("]");
        if i != fixed.len() - 1 {
            toml_string.push_str(",\n");
        }
        toml_string.push_str("\n");
    }
    toml_string.push_str("]\n");
    toml_string
}

pub fn serialize(plaf: Plaf) -> String {
    let base_plaf = toml::to_string(&plaf).expect("Cannot convert to toml");

    let fixed = plaf_fixed_to_toml(plaf.fixed);

    format!("{fixed}\n{base_plaf}")
}

pub fn deserialize(s: String) -> Plaf {
    toml::from_str(&s).unwrap()
}
