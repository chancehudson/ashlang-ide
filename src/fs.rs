use std::collections::HashMap;

use anyhow::Result;

pub fn init() -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    out.insert(
        "pow5.ash".to_string(),
        "(v)

let v2 = v * v
let v4 = v2 * v2

return v4 * v
"
        .to_string(),
    );
    out.insert(
        "assert_eq.ar1cs".to_string(),
        "(a, b) -> ()

# one is a global signal that is equal to 1
# e.g. one === 1
#
# coefficients must be literals
#
# an extra (0,one) is added below
# this evaluates to 0 and is thus a no-op
0 = (1*a + 0*one) * (1*one) - (1*b) # assert equality

# no symbolic constraint necessary
# only operating on known values
"
        .to_string(),
    );
    out.insert(
        "assert_eq.tasm".to_string(),
        "(_, _) -> _

eq
assert
push 0

return
"
        .to_string(),
    );
    out.insert(
        "entry.ash".to_string(),
        "# define some vectors to play with
let x = [1, 2, 3]
let y = [10, 20, 30]

# multiply them together
let m = x * y
assert_eq(m[0], 10)
assert_eq(m[1], 40)
assert_eq(m[2], 90)

# pass the product to a function
let p5 = pow5(m)

# change one of the constants and see what happens!
assert_eq(p5[0], 100000)
assert_eq(p5[1], 102400000)
assert_eq(p5[2], 5904900000)

# the same function can accept scalars
let a5 = pow5(124)
assert_eq(a5, 29316250624)

# click on pow5.ash on the left to see the implementation
"
        .to_string(),
    );
    Ok(out)
}
